//! Resolving a plan's provider refs: which game the target runs and which provider answers
//! each platform.

use crate::features::minecraft::ModLoader;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::SystemTime,
};

use futures::future::try_join_all;

use super::{DownloadEntry, InstallPlanExecutor, ResolvedRef, TargetGame, is_pack_of};
use crate::features::instance::{
    ContentMetadata, ContentProvider, ContentSource, ContentVersionMetadata, InstallFileEntry,
    InstallTarget, InstanceError, PlanOrigin, ProviderId,
};

/// What a plan's direct entries say about their own content, by the provider the content list
/// will look it up under: the one the core picked for the identity's platform, which is not
/// always the provider that gave the plan.
#[derive(Default)]
pub(super) struct PlanMetadata {
    projects: HashMap<ProviderId, Vec<(String, ContentMetadata)>>,
    versions: HashMap<ProviderId, Vec<ContentVersionMetadata>>,
}

impl PlanMetadata {
    /// Take what an entry says about the content it was resolved to. Only what is about the
    /// identity's own ids is taken: the cache is keyed by the provider's ids.
    fn take(
        &mut self,
        resolved: &ResolvedRef,
        metadata: Option<ContentMetadata>,
        version_metadata: Option<ContentVersionMetadata>,
    ) {
        if let Some(metadata) =
            metadata.filter(|metadata| metadata.content_id == resolved.content_id)
        {
            self.projects
                .entry(resolved.provider_id.clone())
                .or_default()
                .push((resolved.content_id.clone(), metadata));
        }
        if let Some(version) =
            version_metadata.filter(|version| version.version_id == resolved.version)
        {
            self.versions
                .entry(resolved.provider_id.clone())
                .or_default()
                .push(version);
        }
    }

    fn extend(&mut self, other: Self) {
        for (provider_id, projects) in other.projects {
            self.projects
                .entry(provider_id)
                .or_default()
                .extend(projects);
        }
        for (provider_id, versions) in other.versions {
            self.versions
                .entry(provider_id)
                .or_default()
                .extend(versions);
        }
    }
}

impl InstallPlanExecutor {
    /// The game the target runs once the plan is applied: the existing instance's own, or the
    /// one the plan moves it to — which only a plan of the instance's pack may do.
    pub(super) async fn target_game(
        &self,
        target: &InstallTarget,
        origin: &PlanOrigin,
    ) -> Result<TargetGame, InstanceError> {
        Ok(match target {
            InstallTarget::Existing { instance_id, game } => {
                let instance = self.instance_storage.get(instance_id).await?;

                match game {
                    None => TargetGame::new(instance.game_version, instance.loader),
                    Some(game) if is_pack_of(&instance, origin) => {
                        TargetGame::new(game.game_version.clone(), game.mod_loader)
                    }
                    Some(_) => {
                        return Err(InstanceError::GameChangeNotAllowed {
                            instance_id: instance_id.to_string(),
                            provider_id: origin.provider_id.clone(),
                            content_id: origin.content_id.clone(),
                        });
                    }
                }
            }
            InstallTarget::New(spec) => TargetGame::new(spec.game_version.clone(), spec.mod_loader),
        })
    }

    /// Turn every entry into a URL to download, keeping its path.
    ///
    /// A ref is resolved by one `resolve_file` call of the provider the core picks for its
    /// platform — the core, never a plugin on another's behalf. One level, by type: a
    /// [`ResolvedFile`](crate::features::instance::ResolvedFile) is a URL, not another ref, so a
    /// ref cannot point at a ref, and neither a chain nor a cycle can form.
    ///
    /// The provider of each platform is picked once per plan, before anything is resolved: a
    /// plan does not resolve half its refs through one provider and fail on the choice for the
    /// rest.
    ///
    /// What the direct entries say about their content is cached once all are resolved.
    pub(super) async fn resolve_all_refs(
        &self,
        files: Vec<InstallFileEntry>,
        origin: &ProviderId,
        game: &TargetGame,
    ) -> Result<Vec<DownloadEntry>, InstanceError> {
        let providers = &self.providers_of(&files).await?;

        let resolved = try_join_all(files.into_iter().map(|file| async move {
            let InstallFileEntry {
                source,
                relative_path,
                size,
                name,
                side,
                metadata,
                version_metadata,
                ..
            } = file;

            match source {
                ContentSource::Direct {
                    url,
                    hash,
                    identity,
                } => {
                    let resolved = identity.and_then(|identity| {
                        providers
                            .get(&identity.platform)
                            .map(|provider| ResolvedRef {
                                provider_id: provider.provider_id(),
                                content_id: identity.content_id,
                                version: identity.version,
                            })
                    });
                    let mut described = PlanMetadata::default();
                    if let Some(resolved) = &resolved {
                        described.take(resolved, metadata, version_metadata);
                    }

                    Ok::<_, InstanceError>((
                        DownloadEntry {
                            relative_path,
                            url,
                            hash,
                            origin: origin.clone(),
                            size,
                            name,
                            side,
                            resolved,
                        },
                        described,
                    ))
                }
                ContentSource::ProviderRef {
                    platform,
                    content_id,
                    version,
                } => {
                    let provider = &providers[&platform];
                    let resolved = provider
                        .resolve_file(
                            &content_id,
                            version.as_deref(),
                            &game.game_version,
                            game.loader.as_ref().map(ModLoader::as_str),
                        )
                        .await?;

                    Ok((
                        DownloadEntry {
                            relative_path,
                            url: resolved.url,
                            hash: resolved.hash,
                            origin: provider.provider_id(),
                            size: resolved.size.or(size),
                            name: name.or(resolved.name),
                            side,
                            resolved: Some(ResolvedRef {
                                provider_id: provider.provider_id(),
                                content_id,
                                version: resolved.version,
                            }),
                        },
                        PlanMetadata::default(),
                    ))
                }
            }
        }))
        .await?;

        let mut entries = Vec::with_capacity(resolved.len());
        let mut described = PlanMetadata::default();
        for (entry, metadata) in resolved {
            entries.push(entry);
            described.extend(metadata);
        }

        // Before any file lands: the list the file watcher's event opens must already find it.
        self.cache_metadata(described).await;

        Ok(entries)
    }

    /// Put what the plan's entries say about their own content into the metadata cache, so a
    /// freshly installed file is listed without asking the provider again. Best effort: a miss
    /// is filled by the list's own lookup.
    ///
    /// A project is put as fetched at the epoch: a provider may give it incomplete (no author
    /// without another request), so it is stale at once, shown until the next list refreshes
    /// it — and only when nothing is cached for it, which may be complete. A version is
    /// complete, and put as fetched now.
    async fn cache_metadata(&self, metadata: PlanMetadata) {
        for (provider_id, projects) in metadata.projects {
            let ids: Vec<String> = projects.iter().map(|(id, _)| id.clone()).collect();
            let cached = self.metadata_cache.get_many(&provider_id, &ids).await;
            let missing: Vec<_> = projects
                .into_iter()
                .filter(|(id, _)| !cached.contains_key(id))
                .collect();
            if missing.is_empty() {
                continue;
            }
            let _ = self
                .metadata_cache
                .put_keyed_at(&provider_id, &missing, SystemTime::UNIX_EPOCH)
                .await;
        }
        for (provider_id, versions) in metadata.versions {
            let _ = self
                .metadata_cache
                .put_versions(&provider_id, &versions)
                .await;
        }
    }

    /// The provider of every platform the plan's files name: by a ref, or by the identity of a
    /// direct source.
    pub(super) async fn providers_of(
        &self,
        files: &[InstallFileEntry],
    ) -> Result<HashMap<String, Arc<dyn ContentProvider>>, InstanceError> {
        let mut ref_platforms = HashSet::new();
        let mut identity_platforms = HashSet::new();
        for file in files {
            match &file.source {
                ContentSource::ProviderRef { platform, .. } => {
                    ref_platforms.insert(platform.as_str());
                }
                ContentSource::Direct {
                    identity: Some(identity),
                    ..
                } => {
                    identity_platforms.insert(identity.platform.as_str());
                }
                ContentSource::Direct { identity: None, .. } => {}
            }
        }

        let mut providers = HashMap::new();
        for platform in &ref_platforms {
            providers.insert(
                (*platform).to_owned(),
                self.provider_for_platform(platform).await?,
            );
        }
        // An identity only enriches a direct file, so a platform nobody serves (or no provider
        // is chosen for) leaves its files unidentified instead of failing the plan.
        for platform in identity_platforms.difference(&ref_platforms) {
            match self.provider_for_platform(platform).await {
                Ok(provider) => {
                    providers.insert((*platform).to_owned(), provider);
                }
                Err(
                    InstanceError::PlatformProviderNotFound { .. }
                    | InstanceError::PlatformProviderNotChosen { .. },
                ) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(providers)
    }

    pub(super) async fn provider_for_platform(
        &self,
        platform: &str,
    ) -> Result<Arc<dyn ContentProvider>, InstanceError> {
        let available = self
            .provider_registry
            .list()
            .await
            .map_err(|_| InstanceError::CapabilityOperationError)?
            .into_iter()
            .map(|entry| entry.capability)
            .collect();
        let selected = self.platform_selection.selected_provider(platform).await?;

        choose_provider(platform, available, selected.as_ref())
    }
}

/// Which of the `available` providers answers a ref to `platform`.
///
/// 1. The one chosen in the settings, if it is available.
/// 2. Without a choice, the only provider serving the platform.
/// 3. Without a choice among several, the built-in one; none built in — the user has to choose.
/// 4. A choice that is not available is an error too, never a quiet switch to another provider.
///
/// A platform no provider serves is an error of its own, with no fallback.
pub(super) fn choose_provider(
    platform: &str,
    available: Vec<Arc<dyn ContentProvider>>,
    selected: Option<&ProviderId>,
) -> Result<Arc<dyn ContentProvider>, InstanceError> {
    let mut serving: Vec<_> = available
        .into_iter()
        .filter(|provider| provider.metadata().platform.as_deref() == Some(platform))
        .collect();

    let not_chosen = || InstanceError::PlatformProviderNotChosen {
        platform: platform.to_owned(),
    };

    if let Some(selected) = selected {
        return serving
            .into_iter()
            .find(|provider| provider.provider_id() == *selected)
            .ok_or_else(|| {
                tracing::warn!(
                    "Provider \"{selected}\" chosen for platform \"{platform}\" is not available"
                );
                not_chosen()
            });
    }

    match serving.len() {
        0 => Err(InstanceError::PlatformProviderNotFound {
            platform: platform.to_owned(),
        }),
        1 => Ok(serving.remove(0)),
        _ => serving
            .into_iter()
            .find(|provider| provider.provider_id().is_builtin())
            .ok_or_else(not_chosen),
    }
}
