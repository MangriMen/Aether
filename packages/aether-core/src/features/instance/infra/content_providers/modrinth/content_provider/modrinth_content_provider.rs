use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt, stream};
use tracing::warn;

use crate::{
    features::{
        instance::{
            CapabilityId, CapabilityMetadata, ContentAuthor, ContentId, ContentIdentity,
            ContentItem, ContentMetadata, ContentPath, ContentProvider,
            ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
            ContentSource, ContentType, ContentVersion, ContentVersionId, ContentVersionMetadata,
            InstallFileEntry, InstallPlan, Instance, InstanceError, InstanceId, InvalidContentId,
            PackInfo, PackMetadata, PackSource, PlatformId, PluginId, ProviderId, ReleaseChannel,
            ResolveInstallPlanParams, ResolveTarget, ResolvedFile,
            app::{
                ContentCompatibilityCheckParams, ContentCompatibilityResult, DownloadEvent,
                DownloadReport,
            },
            infra::content_providers::modrinth::{
                ModrinthMapperError,
                api_client::{
                    File, MODRINTH_API_URL, MODRINTH_API_V3_URL, ModrinthApiClient,
                    ProjectMetadataResponse, ProjectResponse, ProjectSearchParams,
                    ProjectVersionResponse, ProjectVersionsRequest,
                },
            },
            select_version,
        },
        minecraft::{GameVersion, ModLoader},
        settings::LocationInfo,
    },
    shared::io::infra::create_dir_all,
    shared::request_client::{Request, RequestClient},
    shared::web_url::{HttpsUrl, display_url},
};

use super::{
    mapper::{AuthorRef, organization_ref, team_ref},
    mrpack, mrpack_cache,
    version_selection::{
        find_best_version, get_first_file_from_project_version, is_version_compatible,
    },
};

/// How many ids one `GET /projects`, `/versions`, `/teams` or `/organizations` asks for: a hundred keep the
/// URL under 2 KB.
const METADATA_BATCH_SIZE: usize = 100;

/// How many of those requests run at once, well inside the API's 300 requests a minute.
const METADATA_CONCURRENCY: usize = 4;

/// `ids` in their first order, each once.
fn unique(ids: impl Iterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    ids.filter(|id| seen.insert(id.clone())).collect()
}

/// What `fetch` answers for `ids`, asked [`METADATA_BATCH_SIZE`] ids at a time, up to
/// [`METADATA_CONCURRENCY`] requests at once. Any request that fails fails the whole call.
async fn fetch_batched<'a, T, F, Fut>(ids: &'a [String], fetch: F) -> Result<Vec<T>, InstanceError>
where
    F: Fn(&'a [String]) -> Fut,
    Fut: Future<Output = Result<Vec<T>, String>>,
{
    // The requests are made up front: a stream mapping borrowed batches into futures is not
    // `Send` enough for `async_trait`.
    let requests: Vec<_> = ids.chunks(METADATA_BATCH_SIZE).map(fetch).collect();
    let batches: Vec<Vec<T>> = stream::iter(requests)
        .buffer_unordered(METADATA_CONCURRENCY)
        .try_collect()
        .await
        .map_err(InstanceError::ContentDownloadError)?;
    Ok(batches.into_iter().flatten().collect())
}

pub struct ModrinthContentProvider<RC> {
    location_info: Arc<LocationInfo>,
    download_client: Arc<RC>,
    api: ModrinthApiClient<RC>,
    capability: ContentProviderCapabilityMetadata,
    max_mrpack_bytes: u64,
}

impl ModrinthContentProvider<()> {
    pub const ID: &'static str = ProviderId::BUILTIN_PLUGIN_ID;
    pub const CAPABILITY_ID: &'static str = "modrinth";
    /// The platform this provider serves, as [`ContentSource::ProviderRef`] names it.
    ///
    /// [`ContentSource::ProviderRef`]: crate::features::instance::ContentSource::ProviderRef
    pub const PLATFORM: &'static str = "modrinth";

    /// `aether#modrinth`.
    pub fn provider_id() -> ProviderId {
        ProviderId::new(
            PluginId::from_static(Self::ID),
            CapabilityId::from_static(Self::CAPABILITY_ID),
        )
    }
}

impl<RC: RequestClient> ModrinthContentProvider<RC> {
    pub fn new(
        location_info: Arc<LocationInfo>,
        base_headers: Option<reqwest::header::HeaderMap>,
        metadata_client: Arc<RC>,
        download_client: Arc<RC>,
        max_mrpack_bytes: u64,
    ) -> Self {
        let capability = ContentProviderCapabilityMetadata {
            base: CapabilityMetadata {
                id: CapabilityId::from_static(ModrinthContentProvider::CAPABILITY_ID),
                name: "Modrinth".to_string(),
                description: None,
                icon: None,
            },
            platform: Some(PlatformId::from_static(ModrinthContentProvider::PLATFORM)),
        };

        Self {
            download_client,
            api: ModrinthApiClient::new(
                MODRINTH_API_URL.to_string(),
                MODRINTH_API_V3_URL.to_string(),
                base_headers,
                metadata_client,
            ),
            location_info,
            capability,
            max_mrpack_bytes,
        }
    }

    /// The version of a project to install into an instance running `game_version` with
    /// `loader`, by the release channel from the version `installed` now ([`find_best_version`]).
    pub async fn get_project_version_for_game_version(
        &self,
        project_id: &str,
        game_version: &GameVersion,
        loader: Option<&str>,
        installed: Option<&str>,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        let request_params = ProjectVersionsRequest {
            loaders: loader.map(|l| vec![l.to_owned()]),
            game_versions: Some(vec![game_version.to_string()]),
            include_changelog: false,
            ..Default::default()
        };

        let versions = self
            .api
            .get_project_versions(project_id, &request_params)
            .await
            .map_err(InstanceError::ContentDownloadError)?;

        let installed = self
            .installed_version(project_id, installed, &versions)
            .await;

        find_best_version(&versions, game_version, loader, installed.as_ref())
            .cloned()
            .ok_or_else(|| InstanceError::ContentForGameVersionNotFound {
                game_version: game_version.to_string(),
            })
    }

    /// A version of `content_id`, refusing one of another project: a ref or a request naming
    /// a version under the wrong project would record the wrong content for updates.
    ///
    /// `content_id` may be the project's slug, which a version does not carry: the project is
    /// asked for its id then — only for an id that does not match outright.
    async fn version_of(
        &self,
        content_id: &str,
        version_id: &str,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        let version = self
            .api
            .get_project_version(version_id)
            .await
            .map_err(InstanceError::ContentDownloadError)?;

        if version.project_id != content_id
            && self
                .api
                .get_project(content_id)
                .await
                .map_err(InstanceError::ContentDownloadError)?
                .id
                != version.project_id
        {
            return Err(InstanceError::ContentProviderError {
                reason: format!(
                    "Modrinth version {version_id} belongs to project {}, not {content_id}",
                    version.project_id
                ),
            });
        }

        Ok(version)
    }

    /// The version of a pack to install, by the release channel from the version `installed`
    /// now ([`select_version`]) — whatever game it is for: a pack describes its game itself, and
    /// an update may move the instance to another one. This is also what a check for updates
    /// reports.
    async fn pack_version_to_install(
        &self,
        project_id: &str,
        installed: Option<&str>,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        let versions = self.pack_versions(project_id).await?;
        let installed = self
            .installed_version(project_id, installed, &versions)
            .await;

        select_version(&versions, installed.as_ref(), ReleaseChannel::default())
            .cloned()
            .ok_or_else(|| no_pack_versions(project_id))
    }

    /// The installed version of a project, to choose the next one from: found among `versions`,
    /// or asked for when it is not one of them (it does not fit the instance any more). One that
    /// cannot be read is as good as none — the choice falls back to the instance's channel, as
    /// for a first install, rather than failing the install.
    async fn installed_version(
        &self,
        project_id: &str,
        installed: Option<&str>,
        versions: &[ProjectVersionResponse],
    ) -> Option<ProjectVersionResponse> {
        let installed = installed?;
        if let Some(version) = versions.iter().find(|version| version.id == installed) {
            return Some(version.clone());
        }

        self.version_of(project_id, installed)
            .await
            .inspect_err(|err| {
                warn!("Installed version {installed} of {project_id} is not readable: {err}");
            })
            .ok()
    }

    async fn pack_versions(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectVersionResponse>, InstanceError> {
        self.api
            .get_project_versions(project_id, &ProjectVersionsRequest::without_changelog())
            .await
            .map_err(InstanceError::ContentDownloadError)
    }

    fn primary_file(version: &ProjectVersionResponse) -> Result<&File, InstanceError> {
        get_first_file_from_project_version(version).ok_or_else(|| {
            InstanceError::ContentDownloadError(format!(
                "No files found for Modrinth version {}",
                version.id
            ))
        })
    }

    fn resolved_file(version: &ProjectVersionResponse) -> Result<ResolvedFile, InstanceError> {
        let file = Self::primary_file(version)?;

        Ok(ResolvedFile {
            version: response_id::<ContentVersionId>(&version.id)?,
            url: response_url(&file.url)?,
            hash: Some(mrpack::sha512_of(&file.hashes)?),
            file_name: file.filename.clone(),
            size: u64::try_from(file.size).ok(),
            name: Some(version.name.clone()),
        })
    }

    /// The `.mrpack` of a pack version, downloaded once into the cache and verified. The file
    /// is kept: the plan points at it for its overrides, which the executor reads later. Each
    /// call renews it and clears out the archives nobody has used for long (`mrpack_cache`).
    async fn cached_mrpack(
        &self,
        version: &ProjectVersionResponse,
        report: DownloadReport<'_>,
    ) -> Result<PathBuf, InstanceError> {
        let file = Self::primary_file(version)?;
        if !super::mrpack_cache::is_mrpack(Path::new(&file.filename)) {
            return Err(InstanceError::ContentDownloadError(format!(
                "File {} is not a modpack",
                file.filename
            )));
        }

        let url = response_url(&file.url)?;
        let hash = mrpack::sha512_of(&file.hashes)?;
        // Named by the hash: the same archive is downloaded once whoever asks for it, and a file
        // gets this name only once verified against it.
        let path = mrpack_cache::mrpack_dir(&self.location_info.cache_dir())
            .join(format!("{}.mrpack", hash.hex()));
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            mrpack_cache::touch(&path).await;
            self.prune_mrpack_cache(&path).await;
            return Ok(path);
        }

        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }
        if let Ok(size) = u64::try_from(file.size) {
            report(DownloadEvent::Expected(size));
        }
        let on_bytes = |bytes| report(DownloadEvent::Received(bytes));
        self.download_client
            .download_to_file(
                Request::get(url.as_str()).with_hash(hash),
                &path,
                self.max_mrpack_bytes,
                Some(&on_bytes),
            )
            .await
            .map_err(|err| InstanceError::ContentDownloadError(format!("{url}: {err}")))?;
        self.prune_mrpack_cache(&path).await;

        Ok(path)
    }

    /// Clear the cache of archives unused for long, never `current`.
    async fn prune_mrpack_cache(&self, current: &Path) {
        if let Some(dir) = current.parent() {
            mrpack_cache::prune(dir, current, std::time::SystemTime::now()).await;
        }
    }

    async fn resolve_mod_plan(
        &self,
        params: ResolveInstallPlanParams,
        content_type: ContentType,
        project: &ProjectResponse,
    ) -> Result<InstallPlan, InstanceError> {
        let ResolveTarget::Existing {
            game_version,
            loader,
            ..
        } = &params.target
        else {
            return Err(InstanceError::ContentProviderError {
                reason: format!(
                    "{} is a {}, not a modpack: it installs into an existing instance",
                    project.title,
                    content_type.get_name()
                ),
            });
        };

        let version = match &params.version {
            Some(version) => self.version_of(&project.id, version).await?,
            None => {
                self.get_project_version_for_game_version(
                    &project.id,
                    game_version,
                    loader.as_ref().map(ModLoader::as_str),
                    params.installed_version.as_deref(),
                )
                .await?
            }
        };

        let resolved = Self::resolved_file(&version)?;
        let entry = InstallFileEntry {
            size: resolved.size,
            name: resolved.name,
            metadata: Some(Self::project_metadata(project)?),
            version_metadata: Some(ContentVersionMetadata {
                version_id: response_id::<ContentVersionId>(&version.id)?,
                version_number: version.version_number.clone(),
                name: Some(version.name.clone()).filter(|name| !name.is_empty()),
            }),
            ..InstallFileEntry::new(
                ContentSource::Direct {
                    url: resolved.url,
                    hash: resolved.hash,
                    identity: Some(ContentIdentity {
                        platform: PlatformId::from_static(ModrinthContentProvider::PLATFORM),
                        content_id: response_id::<ContentId>(&project.id)?,
                        version: resolved.version,
                    }),
                },
                ContentPath::parse(
                    &content_type
                        .get_relative_path(&resolved.file_name)
                        .to_string_lossy(),
                )?,
                content_type,
            )
        };

        Ok(InstallPlan {
            target: params
                .target
                .into_install_target(|| unreachable!("a new instance is refused above")),
            files: vec![entry],
            overrides: Vec::new(),
        })
    }

    /// What the project says about itself, for the cache, as `get_contents_metadata` gives it:
    /// without an author, which is another request an install does not wait for, but with the
    /// ref the next list asks `get_authors` for it by.
    fn project_metadata(project: &ProjectResponse) -> Result<ContentMetadata, InstanceError> {
        ProjectMetadataResponse::from(project)
            .try_into()
            .map_err(|err| mapper_failure(&err))
    }

    async fn resolve_pack_plan(
        &self,
        params: ResolveInstallPlanParams,
        project: &ProjectResponse,
        report: DownloadReport<'_>,
    ) -> Result<InstallPlan, InstanceError> {
        let version = match &params.version {
            Some(version) => self.version_of(&project.id, version).await?,
            None => {
                self.pack_version_to_install(&project.id, params.installed_version.as_deref())
                    .await?
            }
        };

        let project_id = response_id::<ContentId>(&project.id)?;
        let version_id = response_id::<ContentVersionId>(&version.id)?;
        let archive_path = self.cached_mrpack(&version, report).await?;
        let index = mrpack::read_index_async(&archive_path).await?;

        let metadata = mrpack::pack_metadata(
            &index,
            Some(version_id.clone()),
            display_url("icon of a Modrinth project", project.icon_url.clone()),
        )?;
        let files = mrpack::plan_files(&index)?;
        let target = params.target.into_pack_install_target(&metadata, || {
            Some(PackInfo {
                provider_id: ModrinthContentProvider::provider_id(),
                modpack_id: project_id,
                version_id: version_id.into(),
            })
        })?;

        Ok(InstallPlan {
            target,
            files,
            overrides: mrpack::plan_overrides(&archive_path),
        })
    }

    /// A pack on Modrinth described from the API alone, without downloading its archive: this
    /// is what a check for updates asks, so it stays cheap. The loader version is only in the
    /// archive, and is left out.
    async fn platform_pack_metadata(
        &self,
        content_id: &ContentId,
        version: Option<&ContentVersionId>,
        installed_version: Option<&ContentVersionId>,
    ) -> Result<PackMetadata, InstanceError> {
        let project = self
            .api
            .get_project(content_id)
            .await
            .map_err(InstanceError::ContentDownloadError)?;
        let version = match version {
            Some(version) => self.version_of(&project.id, version.as_str()).await?,
            None => {
                self.pack_version_to_install(
                    &project.id,
                    installed_version.map(ContentVersionId::as_str),
                )
                .await?
            }
        };

        let game_version = GameVersion::parse_list(version.game_versions.clone(), "Modrinth pack")
            .into_iter()
            .next()
            .ok_or_else(|| InstanceError::ContentProviderError {
                reason: format!(
                    "Modrinth version {} names no valid game version",
                    version.id
                ),
            })?;
        let mod_loader = [
            ModLoader::Fabric,
            ModLoader::Forge,
            ModLoader::Quilt,
            ModLoader::NeoForge,
        ]
        .into_iter()
        .find(|loader| version.loaders.iter().any(|name| name == loader.as_str()))
        .unwrap_or(ModLoader::Vanilla);

        Ok(PackMetadata {
            name: project.title,
            version_id: Some(response_id::<ContentVersionId>(&version.id)?),
            version_name: Some(version.version_number),
            description: Some(project.description),
            game_version,
            mod_loader,
            loader_version: None,
            file_count: None,
            icon_url: display_url("icon of a Modrinth project", project.icon_url),
        })
    }

    /// A `.mrpack` at a URL, read once and not kept. One on Modrinth's CDN names its version.
    async fn url_pack_metadata(&self, url: &HttpsUrl) -> Result<PackMetadata, InstanceError> {
        let temp_dir = self.location_info.temp_dir();
        create_dir_all(&temp_dir)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        let path = temp_dir.join(format!("pack-{}.mrpack", uuid::Uuid::new_v4()));

        self.download_client
            .download_to_file(
                Request::get(url.as_str()),
                &path,
                self.max_mrpack_bytes,
                None,
            )
            .await
            .map_err(|err| InstanceError::ContentDownloadError(format!("{url}: {err}")))?;
        let index = mrpack::read_index_async(&path).await;
        let _ = tokio::fs::remove_file(&path).await;

        Ok(PackMetadata {
            version_id: mrpack::parse_cdn_url(url).map(|(_, version_id)| version_id),
            ..mrpack::pack_metadata(&index?, None, None)?
        })
    }
}

#[async_trait]
impl<RC: RequestClient> ContentProvider for ModrinthContentProvider<RC> {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.capability
    }

    fn provider_id(&self) -> ProviderId {
        ModrinthContentProvider::provider_id()
    }

    async fn search(
        &self,
        search_params: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let provider_id = search_params.provider_id.clone();

        let modrinth_search_params =
            ProjectSearchParams::try_from(search_params).map_err(|err| {
                InstanceError::ContentProviderError {
                    reason: format!("Invalid search params: {err}"),
                }
            })?;

        let response = self
            .api
            .search(&modrinth_search_params)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        response.into_content_search(provider_id).map_err(|err| {
            InstanceError::ContentProviderError {
                reason: format!("Invalid search response: {err}"),
            }
        })
    }

    async fn resolve_file(
        &self,
        content_id: &ContentId,
        version: Option<&ContentVersionId>,
        game_version: &GameVersion,
        loader: Option<&str>,
    ) -> Result<ResolvedFile, InstanceError> {
        let version = match version {
            Some(version) => {
                self.version_of(content_id.as_str(), version.as_str())
                    .await?
            }
            None => {
                self.get_project_version_for_game_version(
                    content_id.as_str(),
                    game_version,
                    loader,
                    None,
                )
                .await?
            }
        };

        Self::resolved_file(&version)
    }

    /// A mod (or a resource pack, shader, datapack) is one file of the version fitting the
    /// instance; a modpack is its `.mrpack` — downloaded into the cache, since its files are
    /// listed only inside it — read into the plan's files and overrides.
    ///
    /// Files are direct downloads that carry their identity rather than refs, so the executor
    /// does not ask the API again for each file of a pack.
    async fn resolve_install_plan(
        &self,
        params: ResolveInstallPlanParams,
    ) -> Result<InstallPlan, InstanceError> {
        self.resolve_install_plan_reporting(params, &|_| {}).await
    }

    async fn resolve_install_plan_reporting(
        &self,
        params: ResolveInstallPlanParams,
        report: DownloadReport<'_>,
    ) -> Result<InstallPlan, InstanceError> {
        let project = self
            .api
            .get_project(&params.content_id)
            .await
            .map_err(InstanceError::ContentDownloadError)?;
        let content_type = ContentType::from_string(&project.project_type).ok_or_else(|| {
            InstanceError::ContentProviderError {
                reason: format!("Unknown Modrinth project type: {}", project.project_type),
            }
        })?;

        if content_type == ContentType::Modpack {
            self.resolve_pack_plan(params, &project, report).await
        } else {
            self.resolve_mod_plan(params, content_type, &project).await
        }
    }

    async fn resolve_pack_metadata(
        &self,
        source: PackSource,
    ) -> Result<PackMetadata, InstanceError> {
        match source {
            PackSource::Content {
                content_id,
                version,
                installed_version,
            } => {
                self.platform_pack_metadata(
                    &content_id,
                    version.as_ref(),
                    installed_version.as_ref(),
                )
                .await
            }
            PackSource::Url(url) => self.url_pack_metadata(&url).await,
            PackSource::File(path) => {
                mrpack::pack_metadata(&mrpack::read_index_async(&path).await?, None, None)
            }
        }
    }

    /// Asked of `GET /projects` a batch of ids at a time, a few batches at once. The author is
    /// left to [`Self::get_authors`], by the project's organization or else its team as the
    /// [`ContentMetadata::author_ref`]: a failed request for authors then costs no project its
    /// name and icon.
    ///
    /// A request that fails fails the whole call: a partial answer would read as projects
    /// Modrinth does not know, and be cached as such for days; failed, the caller holds the
    /// lookup back for a few minutes (`LOOKUP_RETRY_AFTER`) before asking again. A project
    /// asked both by its id and its slug is listed once.
    async fn get_contents_metadata(
        &self,
        content_ids: &[ContentId],
    ) -> Result<Vec<ContentMetadata>, InstanceError> {
        let content_ids = unique(content_ids.iter().map(ToString::to_string));
        let batches =
            fetch_batched(&content_ids, |batch| self.api.get_projects_metadata(batch)).await?;
        let mut listed = HashSet::new();
        Ok(batches
            .into_iter()
            .filter(|project| listed.insert(project.id.clone()))
            .filter_map(|project| map_or_skip("project", &project.id.clone(), project))
            .collect())
    }

    /// An organization's ref asked of `GET /organizations`, by the organization's name; a
    /// team's of `GET /teams`, by the username of the team's owner — a batch of ids at a time, a
    /// few batches at once, both at once.
    ///
    /// A request that fails fails the whole call, as in [`Self::get_contents_metadata`]; the
    /// projects keep their names meanwhile. A ref of neither kind, or of a team or organization
    /// Modrinth does not list, is left out.
    async fn get_authors(
        &self,
        author_refs: &[String],
    ) -> Result<Vec<ContentAuthor>, InstanceError> {
        let mut organization_ids = Vec::new();
        let mut team_ids = Vec::new();
        for author_ref in author_refs {
            match AuthorRef::parse(author_ref) {
                Some(AuthorRef::Organization(id)) => organization_ids.push(id.to_owned()),
                Some(AuthorRef::Team(id)) => team_ids.push(id.to_owned()),
                None => {}
            }
        }
        let organization_ids = unique(organization_ids.into_iter());
        let team_ids = unique(team_ids.into_iter());

        let (organizations, teams) = futures::try_join!(
            fetch_batched(&organization_ids, |batch| self.api.get_organizations(batch)),
            fetch_batched(&team_ids, |batch| self.api.get_teams(batch)),
        )?;

        let mut owners: HashMap<String, String> = HashMap::new();
        for member in teams.into_iter().flatten().filter(|member| member.is_owner) {
            owners.entry(member.team_id).or_insert(member.user.username);
        }

        let organizations = organizations.into_iter().map(|organization| ContentAuthor {
            author_ref: organization_ref(&organization.id),
            name: organization.name,
        });
        let owners = owners.into_iter().map(|(team_id, username)| ContentAuthor {
            author_ref: team_ref(&team_id),
            name: username,
        });
        Ok(organizations.chain(owners).collect())
    }

    async fn get_versions_metadata(
        &self,
        version_ids: &[ContentVersionId],
    ) -> Result<Vec<ContentVersionMetadata>, InstanceError> {
        let version_ids = unique(version_ids.iter().map(ToString::to_string));
        let versions =
            fetch_batched(&version_ids, |batch| self.api.get_versions_metadata(batch)).await?;
        Ok(versions
            .into_iter()
            .filter_map(|version| map_or_skip("version", &version.id.clone(), version))
            .collect())
    }

    async fn check_compatibility(
        &self,
        instances: &[Instance],
        check_params: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<InstanceId, ContentCompatibilityResult>, InstanceError> {
        let project = self
            .api
            .get_project(&check_params.content_item.slug)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        let project_versions = self
            .api
            .get_project_versions(&project.id, &ProjectVersionsRequest::without_changelog())
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        let mut compatibility_map = HashMap::new();

        for instance in instances {
            let is_compatible = project_versions
                .iter()
                .any(|version| is_version_compatible(version, &project, instance));

            compatibility_map.insert(
                instance.id().clone(),
                ContentCompatibilityResult { is_compatible },
            );
        }

        Ok(compatibility_map)
    }

    async fn get_content(&self, content_id: ContentId) -> Result<ContentItem, InstanceError> {
        self.api
            .get_project(&content_id)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?
            .try_into()
            .map_err(|err: ModrinthMapperError| {
                InstanceError::ContentDownloadError(err.to_string())
            })
    }

    async fn list_versions(
        &self,
        content_id: ContentId,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        let versions = self
            .api
            .get_project_versions(&content_id, &ProjectVersionsRequest::without_changelog())
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        Ok(versions
            .into_iter()
            .filter_map(|version| map_or_skip("version", &version.id.clone(), version))
            .collect())
    }
}

/// `raw` mapped, or `None` with a warning: one entry Modrinth answered with badly does not take the
/// rest of the answer down, the same way for every list the provider reads.
fn map_or_skip<T, U>(kind: &str, id: &str, raw: T) -> Option<U>
where
    U: TryFrom<T, Error = ModrinthMapperError>,
{
    U::try_from(raw)
        .inspect_err(|err| {
            tracing::warn!("Skipping Modrinth {kind} {id} that failed to map: {err}");
        })
        .ok()
}

fn mapper_failure(err: &ModrinthMapperError) -> InstanceError {
    InstanceError::ContentProviderError {
        reason: err.to_string(),
    }
}

fn response_id<T>(id: &str) -> Result<T, InstanceError>
where
    T: TryFrom<String, Error = InvalidContentId>,
{
    T::try_from(id.to_owned()).map_err(|err| mapper_failure(&err.into()))
}

/// A URL Modrinth answered with, to fetch.
fn response_url(url: &str) -> Result<HttpsUrl, InstanceError> {
    HttpsUrl::parse(url).map_err(|err| mapper_failure(&err.into()))
}

fn no_pack_versions(project_id: &str) -> InstanceError {
    InstanceError::ContentDownloadError(format!("No versions found for modpack {project_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Number(u32);

    impl TryFrom<&str> for Number {
        type Error = ModrinthMapperError;

        fn try_from(raw: &str) -> Result<Self, Self::Error> {
            raw.parse()
                .map(Self)
                .map_err(|_| ModrinthMapperError::MissingField(raw.to_owned()))
        }
    }

    #[test]
    fn should_leave_out_only_the_entries_that_fail_to_map() {
        let mapped: Vec<u32> = ["1", "oops", "3"]
            .into_iter()
            .filter_map(|raw| map_or_skip::<_, Number>("number", raw, raw))
            .map(|number| number.0)
            .collect();

        assert_eq!(mapped, [1, 3]);
    }
}
