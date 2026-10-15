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
            CapabilityMetadata, ContentIdentity, ContentItem, ContentMetadata, ContentProvider,
            ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
            ContentSource, ContentType, ContentVersion, ContentVersionMetadata, InstallFileEntry,
            InstallPlan, Instance, InstanceError, PackInfo, PackMetadata, PackSource, ProviderId,
            ReleaseChannel, ResolveInstallPlanParams, ResolveTarget, ResolvedFile,
            app::{ContentCompatibilityCheckParams, ContentCompatibilityResult},
            infra::content_providers::modrinth::{
                ModrinthMapperError,
                api_client::{
                    File, MODRINTH_API_URL, MODRINTH_API_V3_URL, ModrinthApiClient,
                    ProjectResponse, ProjectSearchParams, ProjectVersionResponse,
                    ProjectVersionsRequest,
                },
            },
            select_version,
        },
        minecraft::ModLoader,
        settings::LocationInfo,
    },
    shared::io::infra::create_dir_all,
    shared::request_client::{Request, RequestClient},
};

use super::{
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

/// Who authors projects: the name of an organization by its id, the username of a team's owner
/// by the team's id.
struct Authors {
    organization_names: HashMap<String, String>,
    owners: HashMap<String, String>,
}

impl Authors {
    /// The organization's name if the project has an organization, else the team's owner; none
    /// when Modrinth does not list it.
    fn of(&self, organization: Option<&str>, team: Option<&str>) -> Option<&str> {
        match (organization, team) {
            (Some(organization), _) => self.organization_names.get(organization),
            (None, Some(team)) => self.owners.get(team),
            (None, None) => None,
        }
        .map(String::as_str)
    }
}

pub struct ModrinthContentProvider<RC> {
    location_info: Arc<LocationInfo>,
    request_client: Arc<RC>,
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
        ProviderId {
            plugin_id: Self::ID.to_owned(),
            capability_id: Self::CAPABILITY_ID.to_owned(),
        }
    }
}

impl<RC: RequestClient> ModrinthContentProvider<RC> {
    pub fn new(
        location_info: Arc<LocationInfo>,
        base_headers: Option<reqwest::header::HeaderMap>,
        request_client: Arc<RC>,
        max_mrpack_bytes: u64,
    ) -> Self {
        let capability = ContentProviderCapabilityMetadata {
            base: CapabilityMetadata {
                id: ModrinthContentProvider::CAPABILITY_ID.to_string(),
                name: "Modrinth".to_string(),
                description: None,
                icon: None,
            },
            platform: Some(ModrinthContentProvider::PLATFORM.to_string()),
        };

        Self {
            request_client: request_client.clone(),
            api: ModrinthApiClient::new(
                MODRINTH_API_URL.to_string(),
                MODRINTH_API_V3_URL.to_string(),
                base_headers,
                request_client,
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
        game_version: &str,
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
                game_version: game_version.to_owned(),
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
            version: version.id.clone(),
            url: file.url.clone(),
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
    ) -> Result<PathBuf, InstanceError> {
        let file = Self::primary_file(version)?;
        if !Path::new(&file.filename)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mrpack"))
        {
            return Err(InstanceError::ContentDownloadError(format!(
                "File {} is not a modpack",
                file.filename
            )));
        }

        let hash = mrpack::sha512_of(&file.hashes)?;
        // Named by the hash: the same archive is downloaded once whoever asks for it, and a file
        // gets this name only once verified against it.
        let path = self
            .location_info
            .cache_dir()
            .join("modrinth")
            .join("mrpack")
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
        self.request_client
            .download_to_file(
                Request::get(&file.url).with_hash(hash),
                &path,
                self.max_mrpack_bytes,
                None,
            )
            .await
            .map_err(|err| InstanceError::ContentDownloadError(format!("{}: {err}", file.url)))?;
        self.prune_mrpack_cache(&path).await;

        Ok(path)
    }

    /// The names of the organizations and the owners of the teams, by id. Any request that
    /// fails fails the whole call.
    async fn authors(
        &self,
        organization_ids: &[String],
        team_ids: &[String],
    ) -> Result<Authors, InstanceError> {
        let (organizations, teams) = futures::try_join!(
            fetch_batched(organization_ids, |batch| self.api.get_organizations(batch)),
            fetch_batched(team_ids, |batch| self.api.get_teams(batch)),
        )?;

        let mut owners: HashMap<String, String> = HashMap::new();
        for member in teams.into_iter().flatten().filter(|member| member.is_owner) {
            owners.entry(member.team_id).or_insert(member.user.username);
        }

        Ok(Authors {
            organization_names: organizations
                .into_iter()
                .map(|organization| (organization.id, organization.name))
                .collect(),
            owners,
        })
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
                    loader.as_deref(),
                    params.installed_version.as_deref(),
                )
                .await?
            }
        };

        let resolved = Self::resolved_file(&version)?;
        let entry = InstallFileEntry {
            size: resolved.size,
            name: resolved.name,
            metadata: Some(Self::project_metadata(project)),
            version_metadata: Some(ContentVersionMetadata {
                version_id: version.id.clone(),
                version_number: version.version_number.clone(),
                name: Some(version.name.clone()).filter(|name| !name.is_empty()),
            }),
            ..InstallFileEntry::new(
                ContentSource::Direct {
                    url: resolved.url,
                    hash: resolved.hash,
                    identity: Some(ContentIdentity {
                        platform: ModrinthContentProvider::PLATFORM.to_owned(),
                        content_id: project.id.clone(),
                        version: resolved.version,
                    }),
                },
                &content_type
                    .get_relative_path(&resolved.file_name)
                    .to_string_lossy(),
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

    /// What the project says about itself, for the cache, named as `get_contents_metadata`
    /// names it, but without an author: that is another request, and an install does not wait
    /// for one. The core caches it as stale, so the next list asks for the author.
    fn project_metadata(project: &ProjectResponse) -> ContentMetadata {
        ContentMetadata {
            content_id: project.id.clone(),
            slug: project.slug.clone(),
            name: project.title.clone(),
            description: Some(project.description.clone()).filter(|text| !text.is_empty()),
            author: None,
            icon_url: project.icon_url.clone().filter(|url| !url.is_empty()),
        }
    }

    async fn resolve_pack_plan(
        &self,
        params: ResolveInstallPlanParams,
        project: &ProjectResponse,
    ) -> Result<InstallPlan, InstanceError> {
        let version = match &params.version {
            Some(version) => self.version_of(&project.id, version).await?,
            None => {
                self.pack_version_to_install(&project.id, params.installed_version.as_deref())
                    .await?
            }
        };

        let archive_path = self.cached_mrpack(&version).await?;
        let index = mrpack::read_index_async(&archive_path).await?;

        let metadata = mrpack::pack_metadata(&index, Some(&version), project.icon_url.clone());
        let files = mrpack::plan_files(&index)?;
        let target = params.target.into_pack_install_target(&metadata, || {
            Some(PackInfo {
                provider_id: ModrinthContentProvider::provider_id(),
                modpack_id: project.id.clone(),
                version_id: version.id.clone(),
            })
        });

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
        content_id: &str,
        version: Option<&str>,
        installed_version: Option<&str>,
    ) -> Result<PackMetadata, InstanceError> {
        let project = self
            .api
            .get_project(content_id)
            .await
            .map_err(InstanceError::ContentDownloadError)?;
        let version = match version {
            Some(version) => self.version_of(&project.id, version).await?,
            None => {
                self.pack_version_to_install(&project.id, installed_version)
                    .await?
            }
        };

        let game_version = version.game_versions.first().cloned().ok_or_else(|| {
            InstanceError::ContentProviderError {
                reason: format!("Modrinth version {} names no game version", version.id),
            }
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
            version_id: Some(version.id),
            version_name: Some(version.version_number),
            description: Some(project.description),
            game_version,
            mod_loader,
            loader_version: None,
            file_count: None,
            icon_url: project.icon_url,
        })
    }

    /// A `.mrpack` at a URL, read once and not kept. One on Modrinth's CDN names its version.
    async fn url_pack_metadata(&self, url: &str) -> Result<PackMetadata, InstanceError> {
        let temp_dir = self.location_info.temp_dir();
        create_dir_all(&temp_dir)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        let path = temp_dir.join(format!("pack-{}.mrpack", uuid::Uuid::new_v4()));

        self.request_client
            .download_to_file(Request::get(url), &path, self.max_mrpack_bytes, None)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(format!("{url}: {err}")))?;
        let index = mrpack::read_index_async(&path).await;
        let _ = tokio::fs::remove_file(&path).await;

        Ok(PackMetadata {
            version_id: mrpack::parse_cdn_url(url).map(|(_, version_id)| version_id),
            ..mrpack::pack_metadata(&index?, None, None)
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

        Ok(response.into_content_search(provider_id))
    }

    async fn resolve_file(
        &self,
        content_id: &str,
        version: Option<&str>,
        game_version: &str,
        loader: Option<&str>,
    ) -> Result<ResolvedFile, InstanceError> {
        let version = match version {
            Some(version) => self.version_of(content_id, version).await?,
            None => {
                self.get_project_version_for_game_version(content_id, game_version, loader, None)
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
            self.resolve_pack_plan(params, &project).await
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
                    version.as_deref(),
                    installed_version.as_deref(),
                )
                .await
            }
            PackSource::Url(url) => self.url_pack_metadata(&url).await,
            PackSource::File(path) => Ok(mrpack::pack_metadata(
                &mrpack::read_index_async(&path).await?,
                None,
                None,
            )),
        }
    }

    /// Asked of `GET /projects` a batch of ids at a time, a few batches at once; the author of
    /// each project then of `GET /organizations` for a project an organization owns, by the
    /// organization's name, and of `GET /teams` for any other, by the username of the team's
    /// owner, batched alike.
    ///
    /// A request that fails fails the whole call: a partial answer would read as projects
    /// Modrinth does not know, or as projects with no author, and be cached as such for days;
    /// failed, the caller holds the lookup back for a few minutes (`LOOKUP_RETRY_AFTER`) before
    /// asking again. A team or organization Modrinth does not list leaves its projects with no
    /// author. A project asked both by its id and its slug is listed once.
    async fn get_contents_metadata(
        &self,
        content_ids: &[String],
    ) -> Result<Vec<ContentMetadata>, InstanceError> {
        let content_ids = unique(content_ids.iter().cloned());
        let batches =
            fetch_batched(&content_ids, |batch| self.api.get_projects_metadata(batch)).await?;
        let mut listed = HashSet::new();
        let projects: Vec<_> = batches
            .into_iter()
            .filter(|project| listed.insert(project.id.clone()))
            .collect();

        let organization_ids = unique(
            projects
                .iter()
                .filter_map(|project| project.organization.clone()),
        );
        let team_ids = unique(
            projects
                .iter()
                .filter(|project| project.organization.is_none())
                .filter_map(|project| project.team.clone()),
        );
        let authors = self.authors(&organization_ids, &team_ids).await?;

        Ok(projects
            .into_iter()
            .map(|project| ContentMetadata {
                author: authors
                    .of(project.organization.as_deref(), project.team.as_deref())
                    .map(str::to_owned),
                ..ContentMetadata::from(project)
            })
            .collect())
    }

    async fn get_versions_metadata(
        &self,
        version_ids: &[String],
    ) -> Result<Vec<ContentVersionMetadata>, InstanceError> {
        let version_ids = unique(version_ids.iter().cloned());
        let versions =
            fetch_batched(&version_ids, |batch| self.api.get_versions_metadata(batch)).await?;
        Ok(versions.into_iter().map(Into::into).collect())
    }

    async fn check_compatibility(
        &self,
        instances: &[Instance],
        check_params: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
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
                instance.id().to_owned(),
                ContentCompatibilityResult { is_compatible },
            );
        }

        Ok(compatibility_map)
    }

    async fn get_content(&self, content_id: String) -> Result<ContentItem, InstanceError> {
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
        content_id: String,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        let versions = self
            .api
            .get_project_versions(&content_id, &ProjectVersionsRequest::without_changelog())
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        Ok(versions
            .into_iter()
            .filter_map(|v| v.try_into().ok()) // Skip versions that failed to map
            .collect())
    }
}

fn no_pack_versions(project_id: &str) -> InstanceError {
    InstanceError::ContentDownloadError(format!("No versions found for modpack {project_id}"))
}
