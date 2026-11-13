use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

use aether_core_plugin_api::v0::{
    ContentAuthorDto, ContentMetadataDto, ContentSearchParamsDto, ContentSearchResultDto,
    ContentVersionDto, ContentVersionMetadataDto, GetAuthorsParamsDto,
    GetContentsMetadataParamsDto, GetVersionsMetadataParamsDto, InstallPlanDto,
    OverrideLocationDto, OverrideSourceDto, PackMetadataDto, PackSourceDto,
    PluginCheckCompatibilityParamsDto, ResolveFileParamsDto, ResolveInstallPlanParamsDto,
    ResolvedFileDto,
};
use async_trait::async_trait;
use extism_convert::Msgpack;
use serde::{Serialize, de::DeserializeOwned};

use crate::features::{
    instance::{
        AnchoredPath, BoxedSource, ContentAuthor, ContentCompatibilityCheckParams,
        ContentCompatibilityResult, ContentId, ContentItem, ContentMetadata, ContentProvider,
        ContentProviderCapabilityMetadata, ContentProviderRejection, ContentSearchParams,
        ContentSearchResult, ContentVersion, ContentVersionId, ContentVersionMetadata,
        InstallFileEntry, InstallPlan, InstallTarget, Instance, InstanceError, InstanceId,
        OverrideLocation, OverrideSource, PackMetadata, PackSource, PageIndex, PageSize, PluginId,
        ProviderId, ResolveInstallPlanParams, ResolveTarget, ResolvedFile,
    },
    minecraft::GameVersion,
    plugins::{
        PluginCheckCompatibilityParams, PluginContentProviderCapability, PluginError, PluginHandle,
    },
    settings::LocationInfo,
};

use crate::features::plugins::infra::extism::models::PluginInstanceExt;
use crate::features::plugins::infra::extism::{InvalidPlanFile, InvalidResolvedFile};
use crate::features::plugins::infra::pack_instances::deploying_packs;
use crate::features::plugins::infra::plugin_utils::{
    MountSet, host_path_to_plugin, parse_plugin_path, resolve_plugin_path,
    resolve_plugin_path_anchored,
};
use crate::shared::web_url::display_icon_url;

/// A content provider implemented by a plugin, calling the functions its `capabilities.json`
/// names.
///
/// The plugin boundary for what a provider returns: hashes are validated, a plan has to answer
/// the request, and every plugin-side path is translated or refused, so nothing past the proxy
/// sees one.
pub struct PluginContentProviderProxy {
    plugin_id: PluginId,
    provider_id: ProviderId,
    instance: PluginHandle,
    capability: PluginContentProviderCapability,
    location_info: Arc<LocationInfo>,
}

impl PluginContentProviderProxy {
    pub fn new(
        plugin_id: PluginId,
        instance: PluginHandle,
        capability: PluginContentProviderCapability,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        let provider_id = ProviderId::new(plugin_id.clone(), capability.metadata.base.id.clone());

        Self {
            plugin_id,
            provider_id,
            instance,
            capability,
            location_info,
        }
    }

    /// The handler of an operation a plugin may leave out. Without one the operation is
    /// unsupported, exactly as for a built-in provider that keeps the trait's default.
    fn handler<'a>(
        &self,
        handler: Option<&'a String>,
        operation: &'static str,
    ) -> Result<&'a str, InstanceError> {
        handler
            .map(String::as_str)
            .ok_or_else(|| self.unsupported(operation))
    }

    /// The error for an answer that decoded but cannot be accepted; a wrong shape already fails
    /// while decoding.
    fn invalid_answer(&self, handler_name: &str, source: impl Into<BoxedSource>) -> InstanceError {
        let source = source.into();
        tracing::error!(
            "Plugin '{}' returned an invalid answer from '{}': {}",
            self.plugin_id,
            handler_name,
            source
        );

        // A refused URL is told as such: its text would carry the query, tokens included, to
        // the user, where the URL is shown without it.
        let invalid_url = source
            .downcast_ref::<InvalidPlanFile>()
            .and_then(InvalidPlanFile::invalid_url)
            .or_else(|| {
                source
                    .downcast_ref::<InvalidResolvedFile>()
                    .and_then(InvalidResolvedFile::invalid_url)
            });
        if let Some(url) = invalid_url {
            return ContentProviderRejection::ResponseInvalidUrl(url.clone()).into();
        }

        ContentProviderRejection::InvalidAnswer {
            handler: handler_name.to_owned(),
            source: source.into(),
        }
        .into()
    }

    /// An answer's items without what was not asked for, what is described too poorly to show
    /// (`usable`), and what repeats an earlier item (`id`): dropped and told, not an error.
    /// One bad item must not cost the others, and a plugin cannot fill the cache with what
    /// nobody asked about. `asked` tells whether an item answers one of the ids asked.
    fn keep_answered<T>(
        &self,
        handler_name: &str,
        items: Vec<T>,
        asked: impl Fn(&T) -> bool,
        usable: impl Fn(&T) -> bool,
        id: impl Fn(&T) -> &str,
    ) -> Vec<T> {
        let total = items.len();
        let mut seen = HashSet::new();
        let kept: Vec<T> = items
            .into_iter()
            .filter(|item| asked(item) && usable(item) && seen.insert(id(item).to_owned()))
            .collect();
        if kept.len() != total {
            tracing::warn!(
                "Plugin '{}' answered '{}' with {total} items, dropped {} of them as unasked,                  repeated or too poorly described",
                self.plugin_id,
                handler_name,
                total - kept.len()
            );
        }
        kept
    }

    fn install_plan_from_dto(
        &self,
        handler_name: &str,
        requested: &ResolveTarget,
        dto: InstallPlanDto,
    ) -> Result<InstallPlan, InstanceError> {
        let mut target = InstallTarget::try_from(dto.target)?;
        if !requested.is_answered_by(&target) {
            tracing::error!(
                "Plugin '{}' answered '{handler_name}' with a plan for {}, but {} was asked for",
                self.plugin_id,
                describe_install_target(&target),
                describe_resolve_target(requested)
            );
            return Err(ContentProviderRejection::WrongTarget {
                provider_id: self.provider_id.clone(),
            }
            .into());
        }

        // Held to the plugin's cache like the overrides: a host path would let the plugin show
        // any file of the user's as the instance icon. Unlike them it is passed on as a plain
        // host path and read with links followed, so a link swapped in after this check is
        // still followed (see `resolve_plugin_path`).
        if let InstallTarget::New(instance) = &mut target
            && let Some(icon_path) = instance.icon_path.take()
        {
            let host_path = self
                .cache_file_to_host(&icon_path)
                .map_err(|err| self.invalid_answer(handler_name, err))?;
            instance.icon_path = Some(host_path.to_string_lossy().into_owned());
        }

        let files = dto
            .files
            .into_iter()
            .map(InstallFileEntry::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| self.invalid_answer(handler_name, err))?;

        let overrides = dto
            .overrides
            .into_iter()
            .map(|overrides| self.override_source_from_dto(overrides))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| self.invalid_answer(handler_name, err))?;

        Ok(InstallPlan {
            target,
            files,
            overrides,
        })
    }

    /// The host file behind `path`, a path the plugin sent that must name a file of its own
    /// `/cache` (see [`resolve_plugin_path`]).
    fn cache_file_to_host(&self, path: &str) -> Result<PathBuf, PluginError> {
        let path = parse_plugin_path(&self.plugin_id, path)?;
        resolve_plugin_path(&self.location_info, &self.plugin_id, &path, MountSet::CACHE)
    }

    /// Override paths name files in the plugin's own `/cache` and nothing else. They stay
    /// anchored at the root of `/cache`: the plugin writes there, so the core opens what is under
    /// it without following a link the plugin may put in after this check.
    fn override_source_from_dto(
        &self,
        dto: OverrideSourceDto,
    ) -> Result<OverrideSource, PluginError> {
        let location = match dto.location {
            OverrideLocationDto::Archive {
                archive_path,
                subdir,
            } => OverrideLocation::Archive {
                archive_path: self.cache_file_anchored(&archive_path)?,
                subdir,
            },
            OverrideLocationDto::Directory { path } => OverrideLocation::Directory {
                path: self.cache_file_anchored(&path)?,
            },
        };

        Ok(OverrideSource::new(location, dto.side.into()))
    }

    /// [`Self::cache_file_to_host`], anchored at the root of the plugin's `/cache`.
    fn cache_file_anchored(&self, path: &str) -> Result<AnchoredPath, PluginError> {
        let plugin_path = parse_plugin_path(&self.plugin_id, path)?;
        let (root, relative) = resolve_plugin_path_anchored(
            &self.location_info,
            &self.plugin_id,
            &plugin_path,
            MountSet::CACHE,
        )?;

        AnchoredPath::new(root, relative).map_err(|err| {
            tracing::debug!(
                "Plugin '{}' sent a path the core refuses: {err}",
                self.plugin_id
            );
            PluginError::AccessViolation {
                plugin_id: self.plugin_id.clone(),
                path: path.to_owned(),
            }
        })
    }

    /// A pack file is handed over under the plugin's own name for it; a file the plugin cannot
    /// read (outside `/cache` and `/instances`) is refused instead of passed on as a path that
    /// would fail inside the plugin.
    fn pack_source_to_dto(&self, source: PackSource) -> Result<PackSourceDto, InstanceError> {
        Ok(match source {
            PackSource::Content {
                content_id,
                version,
                installed_version,
            } => PackSourceDto::Content {
                content_id: content_id.into(),
                version: version.map(Into::into),
                installed_version: installed_version.map(Into::into),
            },
            PackSource::Url(url) => PackSourceDto::Url(url.into()),
            PackSource::File(path) => PackSourceDto::File(
                host_path_to_plugin(&self.location_info, &self.plugin_id, &path).ok_or_else(
                    || InstanceError::ContentProviderError {
                        reason: ContentProviderRejection::PackOutsideProviderReach {
                            path: path.clone(),
                        },
                    },
                )?,
            ),
        })
    }

    async fn call_plugin<I, O>(
        &self,
        operation: &'static str,
        handler_name: &str,
        input: I,
    ) -> Result<O, InstanceError>
    where
        I: Serialize + Send + 'static,
        O: DeserializeOwned + Send + 'static,
    {
        self.call_plugin_with(operation, handler_name, input, false)
            .await
    }

    /// `deploys_pack`: the call creates and fills an instance for a pack, which stays
    /// `PackInstalling` until the call is over (`infra::pack_instances`).
    async fn call_plugin_with<I, O>(
        &self,
        operation: &'static str,
        handler_name: &str,
        input: I,
        deploys_pack: bool,
    ) -> Result<O, InstanceError>
    where
        I: Serialize + Send + 'static,
        O: DeserializeOwned + Send + 'static,
    {
        let handler = handler_name.to_owned();
        let plugin_id = self.plugin_id.clone();
        let result = self
            .instance
            .run(handler_name, move |plugin| {
                plugin.supports(&handler).then(|| {
                    let call = || {
                        plugin
                            .call::<Msgpack<I>, Msgpack<O>>(&handler, Msgpack(input))
                            .map(|res| res.0)
                    };
                    if deploys_pack {
                        deploying_packs(&plugin_id, call).0
                    } else {
                        call()
                    }
                })
            })
            .await
            .and_then(Option::transpose);

        match result {
            Ok(Some(output)) => Ok(output),
            Ok(None) => {
                tracing::error!(
                    "Plugin '{}' missing handler '{}' for capability '{}'",
                    self.plugin_id,
                    handler_name,
                    self.capability.id
                );
                Err(self.unsupported(operation))
            }
            Err(PluginError::Stopped { plugin_id }) => Err(InstanceError::PluginNotRunning {
                plugin_id: plugin_id.into(),
            }),
            Err(err) => {
                tracing::error!(
                    "Error calling plugin '{}' ({}): {:?}",
                    self.plugin_id,
                    handler_name,
                    err
                );
                Err(InstanceError::ContentProviderError {
                    reason: ContentProviderRejection::Provider(err.to_string()),
                })
            }
        }
    }
}

#[async_trait]
impl ContentProvider for PluginContentProviderProxy {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.capability
    }

    fn provider_id(&self) -> ProviderId {
        self.provider_id.clone()
    }

    async fn search(
        &self,
        search_content: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let dto: ContentSearchParamsDto = search_content.into();
        let result_dto: ContentSearchResultDto = self
            .call_plugin("search", &self.capability.handlers.search, dto)
            .await?;
        Ok(ContentSearchResult {
            page: PageIndex::parse(i64::from(result_dto.page))
                .map_err(|err| self.invalid_answer("search", err))?,
            page_size: PageSize::parse(i64::from(result_dto.page_size))
                .map_err(|err| self.invalid_answer("search", err))?,
            page_count: result_dto.page_count,
            provider_id: result_dto
                .provider_id
                .try_into()
                .map_err(|err| self.invalid_answer("search", err))?,
            items: result_dto
                .items
                .into_iter()
                .map(ContentItem::from)
                .collect(),
        })
    }

    async fn get_content(&self, content_id: ContentId) -> Result<ContentItem, InstanceError> {
        let result_dto: aether_core_plugin_api::v0::ContentItemDto = self
            .call_plugin(
                "get_content",
                &self.capability.handlers.get_content,
                String::from(content_id),
            )
            .await?;
        Ok(result_dto.into())
    }

    async fn list_versions(
        &self,
        content_id: ContentId,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.list_version.as_ref(),
            "list_versions",
        )?;
        let result_dto: Vec<ContentVersionDto> = self
            .call_plugin("list_versions", handler, String::from(content_id))
            .await?;
        result_dto
            .into_iter()
            .map(ContentVersion::try_from)
            .collect::<Result<_, _>>()
            .map_err(|err| self.invalid_answer("list_versions", err))
    }

    async fn get_version(
        &self,
        version_id: &ContentVersionId,
    ) -> Result<ContentVersion, InstanceError> {
        let handler = self.handler(self.capability.handlers.get_version.as_ref(), "get_version")?;
        let result_dto: ContentVersionDto = self
            .call_plugin("get_version", handler, version_id.as_str().to_owned())
            .await?;
        ContentVersion::try_from(result_dto).map_err(|err| self.invalid_answer("get_version", err))
    }

    async fn resolve_file(
        &self,
        content_id: &ContentId,
        version: Option<&ContentVersionId>,
        game_version: &GameVersion,
        loader: Option<&str>,
    ) -> Result<ResolvedFile, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.resolve_file.as_ref(),
            "resolve_file",
        )?;
        let params = ResolveFileParamsDto {
            content_id: content_id.to_string(),
            version: version.map(ToString::to_string),
            game_version: game_version.to_string(),
            loader: loader.map(ToOwned::to_owned),
        };

        let result_dto: ResolvedFileDto = self.call_plugin("resolve_file", handler, params).await?;
        result_dto
            .try_into()
            .map_err(|err| self.invalid_answer(handler, err))
    }

    async fn resolve_install_plan(
        &self,
        params: ResolveInstallPlanParams,
        _: crate::features::instance::DownloadReport<'_>,
    ) -> Result<InstallPlan, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.resolve_install_plan.as_ref(),
            "resolve_install_plan",
        )?;
        let requested = params.target.clone();

        let result_dto: InstallPlanDto = self
            .call_plugin(
                "resolve_install_plan",
                handler,
                ResolveInstallPlanParamsDto::from(params),
            )
            .await?;
        self.install_plan_from_dto(handler, &requested, result_dto)
    }

    async fn resolve_pack_metadata(
        &self,
        source: PackSource,
    ) -> Result<PackMetadata, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.resolve_pack_metadata.as_ref(),
            "resolve_pack_metadata",
        )?;
        let source = self.pack_source_to_dto(source)?;

        let result_dto: PackMetadataDto = self
            .call_plugin("resolve_pack_metadata", handler, source)
            .await?;
        result_dto
            .try_into()
            .map_err(|err| self.invalid_answer("resolve_pack_metadata", err))
    }

    async fn get_contents_metadata(
        &self,
        content_ids: &[ContentId],
    ) -> Result<Vec<ContentMetadata>, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.get_contents_metadata.as_ref(),
            "get_contents_metadata",
        )?;

        let result_dto: Vec<ContentMetadataDto> = self
            .call_plugin(
                "get_contents_metadata",
                handler,
                GetContentsMetadataParamsDto {
                    content_ids: content_ids.iter().map(ToString::to_string).collect(),
                },
            )
            .await?;
        // An item answers an id by its own id or by its slug, as a provider may be asked by
        // either. A slug match is kept under the item's own id, which the host cannot check
        // against what was asked; it stays within the plugin's own provider, and no more than
        // one item per asked id is kept.
        let asked = |dto: &ContentMetadataDto| {
            content_ids.iter().any(|id| id.as_str() == dto.content_id)
                || content_ids
                    .iter()
                    .any(|id| id.eq_ignore_ascii_case(&dto.slug))
        };
        Ok(self
            .keep_answered(
                handler,
                result_dto,
                asked,
                |dto| {
                    ContentId::try_from(dto.content_id.as_str()).is_ok()
                        && !dto.name.trim().is_empty()
                },
                |dto| &dto.content_id,
            )
            .into_iter()
            .take(content_ids.len())
            .filter_map(content_metadata_from_dto)
            .map(|mut metadata| {
                // A ref no handler can resolve would only fail every lookup of it.
                if self.capability.handlers.get_authors.is_none() {
                    metadata.author_ref = None;
                }
                metadata
            })
            .collect())
    }

    async fn get_authors(
        &self,
        author_refs: &[String],
    ) -> Result<Vec<ContentAuthor>, InstanceError> {
        let handler = self.handler(self.capability.handlers.get_authors.as_ref(), "get_authors")?;

        let result_dto: Vec<ContentAuthorDto> = self
            .call_plugin(
                "get_authors",
                handler,
                GetAuthorsParamsDto {
                    author_refs: author_refs.to_vec(),
                },
            )
            .await?;
        let asked: HashSet<&str> = author_refs.iter().map(String::as_str).collect();
        Ok(self
            .keep_answered(
                handler,
                result_dto,
                |dto| asked.contains(dto.author_ref.as_str()),
                |dto| !dto.name.trim().is_empty(),
                |dto| &dto.author_ref,
            )
            .into_iter()
            .map(|dto| ContentAuthor {
                author_ref: dto.author_ref,
                name: author_name(&dto.name),
            })
            .collect())
    }

    async fn get_versions_metadata(
        &self,
        version_ids: &[ContentVersionId],
    ) -> Result<Vec<ContentVersionMetadata>, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.get_versions_metadata.as_ref(),
            "get_versions_metadata",
        )?;

        let result_dto: Vec<ContentVersionMetadataDto> = self
            .call_plugin(
                "get_versions_metadata",
                handler,
                GetVersionsMetadataParamsDto {
                    version_ids: version_ids.iter().map(ToString::to_string).collect(),
                },
            )
            .await?;
        let asked: HashSet<&str> = version_ids.iter().map(ContentVersionId::as_str).collect();
        Ok(self
            .keep_answered(
                handler,
                result_dto,
                |dto| asked.contains(dto.version_id.as_str()),
                |dto| {
                    ContentVersionId::try_from(dto.version_id.as_str()).is_ok()
                        && !dto.version_number.trim().is_empty()
                },
                |dto| &dto.version_id,
            )
            .into_iter()
            .filter_map(version_metadata_from_dto)
            .collect())
    }

    async fn check_compatibility(
        &self,
        instances: &[Instance],
        check_params: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<InstanceId, ContentCompatibilityResult>, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.check_compatibility.as_ref(),
            "check_compatibility",
        )?;
        let params: PluginCheckCompatibilityParamsDto = PluginCheckCompatibilityParams {
            instances: instances.to_vec(),
            check_params: check_params.clone(),
        }
        .into();

        let result_dto: HashMap<String, aether_core_plugin_api::v0::ContentCompatibilityResultDto> =
            self.call_plugin("check_compatibility", handler, params)
                .await?;
        result_dto
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    InstanceId::try_from(k)?,
                    ContentCompatibilityResult {
                        is_compatible: v.is_compatible,
                    },
                ))
            })
            .collect()
    }
}

fn describe_install_target(target: &InstallTarget) -> String {
    match target {
        InstallTarget::Existing { instance_id, .. } => format!("instance '{instance_id}'"),
        InstallTarget::New(_) => "a new instance".to_owned(),
    }
}

fn describe_resolve_target(target: &ResolveTarget) -> String {
    match target {
        ResolveTarget::Existing { instance_id, .. } => format!("instance '{instance_id}'"),
        ResolveTarget::New => "a new instance".to_owned(),
    }
}

/// An icon is taken only as an `https` address of a public host, which the host fetches itself:
/// anything else is dropped, with a warning, so the plugin cannot point the host or the UI at a
/// file of the user's or a machine of their network.
fn content_metadata_from_dto(dto: ContentMetadataDto) -> Option<ContentMetadata> {
    let icon_url = display_icon_url("icon of a plugin's project", dto.icon_url);

    Some(ContentMetadata {
        content_id: dto.content_id.try_into().ok()?,
        slug: dto.slug,
        name: dto.name,
        author: dto.author,
        author_ref: dto.author_ref.filter(|author_ref| {
            !author_ref.trim().is_empty() && author_ref.len() <= MAX_AUTHOR_REF_LEN
        }),
        icon_url,
    })
}

/// The longest ref the host keeps and sends back to a plugin; it is an opaque token (a team or
/// organization id), so a longer one is a plugin's mistake, not an author.
const MAX_AUTHOR_REF_LEN: usize = 256;
const MAX_AUTHOR_NAME_CHARS: usize = 128;

/// An author's name is shown as one line, so it is trimmed and cut to a length that fits.
fn author_name(name: &str) -> String {
    name.trim().chars().take(MAX_AUTHOR_NAME_CHARS).collect()
}

/// An empty title is no title.
fn version_metadata_from_dto(dto: ContentVersionMetadataDto) -> Option<ContentVersionMetadata> {
    Some(ContentVersionMetadata {
        version_id: dto.version_id.try_into().ok()?,
        version_number: dto.version_number,
        name: dto.name.filter(|name| !name.trim().is_empty()),
    })
}
