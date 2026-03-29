use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use futures::StreamExt;
use tracing::warn;
use url::Url;

use crate::{
    features::{
        instance::{
            app::{ContentCompatibilityCheckParams, ContentCompatibilityResult, NewInstance},
            infra::content_providers::modrinth::api_client::{
                File, ModrinthApiClient, ModrinthIndex, ModrinthIndexFile, ProjectSearchParams,
                ProjectVersionResponse, ProjectVersionsRequest, MODRINTH_API_URL,
            },
            AtomicInstallParams, CapabilityMetadata, ContentFile, ContentProvider,
            ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
            ContentType, CreateContentFileParams, Instance, InstanceError, ModpackInstallParams,
            PackInfo, ProviderId,
        },
        minecraft::LoaderVersionPreference,
        settings::LocationInfo,
    },
    libs::request_client::{Request, RequestClient},
    shared::{create_dir_all, write_async},
};

use super::{find_best_version, get_first_file_from_project_version, is_version_compatible};

pub struct ModrinthContentProvider<RC> {
    location_info: Arc<LocationInfo>,
    request_client: Arc<RC>,
    api: ModrinthApiClient<RC>,
    capability: ContentProviderCapabilityMetadata,
}

impl ModrinthContentProvider<()> {
    pub const ID: &'static str = "core:modrinth";
}

impl<RC: RequestClient> ModrinthContentProvider<RC> {
    pub fn new(
        location_info: Arc<LocationInfo>,
        base_headers: Option<reqwest::header::HeaderMap>,
        request_client: Arc<RC>,
    ) -> Self {
        let capability = ContentProviderCapabilityMetadata {
            base: CapabilityMetadata {
                id: "modrinth-content".to_string(),
                name: "Modrinth".to_string(),
                description: None,
                icon: None,
            },
            supports_install_atomic: true,
            supports_install_modpacks: true,
        };

        Self {
            request_client: request_client.clone(),
            api: ModrinthApiClient::new(MODRINTH_API_URL.to_string(), base_headers, request_client),
            location_info,
            capability,
        }
    }

    fn get_provider_id(&self) -> ProviderId {
        ProviderId {
            plugin_id: ModrinthContentProvider::ID.to_owned(),
            capability_id: self.capability.base.id.clone(),
        }
    }

    pub async fn get_project_version_for_game_version(
        &self,
        project_id: &str,
        game_version: &str,
        loader: &Option<String>,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        let request_params = ProjectVersionsRequest {
            loaders: loader.as_ref().map(|l| vec![l.clone()]),
            game_versions: Some(vec![game_version.to_string()]),
            include_changelog: Some(false),
            ..Default::default()
        };

        let versions = self
            .api
            .get_project_versions(project_id, &request_params)
            .await
            .map_err(InstanceError::ContentDownloadError)?;

        find_best_version(&versions, game_version, loader)
            .cloned()
            .ok_or_else(|| InstanceError::ContentForGameVersionNotFound {
                game_version: game_version.to_owned(),
            })
    }

    async fn resolve_project_version(
        &self,
        install_params: &AtomicInstallParams,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        match &install_params.content_version {
            Some(version) => self
                .api
                .get_project_version(version)
                .await
                .map_err(|err| InstanceError::ContentDownloadError(err.to_string())),
            None => {
                self.get_project_version_for_game_version(
                    &install_params.content_id,
                    &install_params.game_version,
                    &install_params.loader,
                )
                .await
            }
        }
    }

    fn get_project_file<'a>(
        project_version: &'a ProjectVersionResponse,
        install_params: &AtomicInstallParams,
    ) -> Result<&'a File, InstanceError> {
        get_first_file_from_project_version(project_version).ok_or(
            InstanceError::ContentForGameVersionNotFound {
                game_version: install_params.game_version.to_owned(),
            },
        )
    }

    async fn fetch_to_disk(&self, url: &str, path: &Path) -> Result<(), InstanceError> {
        let file_bytes = self
            .request_client
            .fetch_bytes(Request::get(url))
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        Ok(write_async(path, &file_bytes).await?)
    }

    fn get_absolute_content_path(&self, instance_id: &str, relative_path: &Path) -> PathBuf {
        self.location_info
            .instance_dir(instance_id)
            .join(relative_path)
    }

    async fn resolve_modpack_version(
        &self,
        install_params: &ModpackInstallParams,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        match &install_params.content_version {
            Some(version_id) => self
                .api
                .get_project_version(version_id)
                .await
                .map_err(|err| InstanceError::ContentDownloadError(err.to_string())),
            None => {
                let versions = self
                    .api
                    .get_project_versions(
                        &install_params.content_id,
                        &ProjectVersionsRequest::without_changelog(),
                    )
                    .await
                    .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

                versions
                    .first()
                    .cloned()
                    .ok_or(InstanceError::ContentDownloadError(
                        "No versions found for modpack".to_string(),
                    ))
            }
        }
    }

    fn read_modpack_manifest(mrpack_path: &Path) -> Result<ModrinthIndex, InstanceError> {
        let file =
            std::fs::File::open(mrpack_path).map_err(|e| InstanceError::ContentProviderError {
                reason: format!("Failed to open .mrpack file: {e}"),
            })?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| InstanceError::ContentProviderError {
                reason: format!("Invalid .mrpack archive: {e}"),
            })?;

        const INDEX_PATH: &str = "modrinth.index.json";

        let mut index_file =
            archive
                .by_name(INDEX_PATH)
                .map_err(|_| InstanceError::ContentProviderError {
                    reason: format!("Critical: {INDEX_PATH} missing in modpack"),
                })?;

        serde_json::from_reader(&mut index_file).map_err(|e| InstanceError::ContentProviderError {
            reason: format!("Malformed modpack index: {e}"),
        })
    }

    fn parse_file_url(url_str: &str) -> Option<(String, String)> {
        let parsed_url = Url::parse(url_str).ok()?;
        let segments: Vec<&str> = parsed_url.path_segments()?.collect();

        // URL: https://cdn.modrinth.com/data/AABBCCDD/versions/EEFFGGHH/file.jar
        let data_idx = segments.iter().position(|&s| s == "data")?;
        let versions_idx = segments.iter().position(|&s| s == "versions")?;

        if versions_idx == data_idx + 2 && segments.len() > versions_idx + 1 {
            let project_id = segments[data_idx + 1].to_string();
            let version_id = segments[versions_idx + 1].to_string();
            return Some((project_id, version_id));
        }

        None
    }

    fn extract_overrides(mrpack_path: &Path, instance_dir: &Path) -> Result<(), InstanceError> {
        let file =
            std::fs::File::open(mrpack_path).map_err(|e| InstanceError::ContentProviderError {
                reason: format!("Failed to open pack: {e}"),
            })?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| InstanceError::ContentProviderError {
                reason: format!("Invalid zip: {e}"),
            })?;

        const MODPACK_OVERRIDES_DIR: &str = "overrides/";

        for i in 0..archive.len() {
            let mut zip_file =
                archive
                    .by_index(i)
                    .map_err(|e| InstanceError::ContentProviderError {
                        reason: format!("Failed to read zip entry {i}: {e}"),
                    })?;

            let entry_path = match zip_file.enclosed_name() {
                Some(path) if path.starts_with(MODPACK_OVERRIDES_DIR) => {
                    path.strip_prefix(MODPACK_OVERRIDES_DIR).unwrap().to_owned()
                }
                _ => continue,
            };

            let full_path = instance_dir.join(&entry_path);

            if zip_file.is_dir() {
                std::fs::create_dir_all(&full_path).map_err(|e| {
                    InstanceError::ContentProviderError {
                        reason: format!("Failed to create directory {:?}: {e}", full_path),
                    }
                })?;
                continue;
            }

            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    InstanceError::ContentProviderError {
                        reason: format!(
                            "Failed to create parent directory for {:?}: {e}",
                            full_path
                        ),
                    }
                })?;
            }

            let mut outfile = std::fs::File::create(&full_path).map_err(|e| {
                InstanceError::ContentProviderError {
                    reason: format!("Failed to create file {:?}: {e}", full_path),
                }
            })?;

            std::io::copy(&mut zip_file, &mut outfile).map_err(|e| {
                InstanceError::ContentProviderError {
                    reason: format!("Failed to copy data to {:?}: {e}", full_path),
                }
            })?;
        }

        Ok(())
    }

    async fn download_modpack_file(
        &self,
        instance_dir: PathBuf,
        mod_file: ModrinthIndexFile,
    ) -> Result<Option<ContentFile>, InstanceError> {
        let url = match mod_file.downloads.first() {
            Some(u) => u,
            None => return Ok(None),
        };

        let (project_id, version_id) = match Self::parse_file_url(url) {
            Some(ids) => ids,
            None => return Ok(None),
        };

        let content_path = PathBuf::from(&mod_file.path);
        let content_type = match ContentType::get_from_parent_folder(&content_path) {
            Some(t) => t,
            None => return Ok(None),
        };

        let target_path = instance_dir.join(&content_path);

        if let Some(parent) = target_path.parent() {
            create_dir_all(parent).await?;
        }

        self.fetch_to_disk(url, &target_path).await?;

        Ok(Some(ContentFile::from_params(CreateContentFileParams {
            name: None,
            file_name: content_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            size: mod_file.file_size,
            sha1: mod_file.hashes.sha1.clone(),
            content_path,
            content_id: project_id,
            content_version: version_id,
            content_type,
            provider_id: self.get_provider_id(),
        })))
    }

    async fn deploy_modpack_files(
        &self,
        mrpack_path: &Path,
        instance_id: &str,
        manifest: ModrinthIndex,
    ) -> Result<Vec<ContentFile>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        let path_sync = mrpack_path.to_owned();
        let dir_sync = instance_dir.clone();
        tokio::task::spawn_blocking(move || Self::extract_overrides(&path_sync, &dir_sync))
            .await
            .map_err(|e| InstanceError::ContentProviderError {
                reason: e.to_string(),
            })??;

        let processed_files: Vec<ContentFile> = futures::stream::iter(manifest.files)
            .map(|mod_file| {
                let dir = instance_dir.clone();

                async move { self.download_modpack_file(dir, mod_file).await }
            })
            .buffer_unordered(5)
            .filter_map(|res| async {
                match res {
                    Ok(Some(file)) => Some(file),
                    Ok(None) => None,
                    Err(e) => {
                        warn!("Failed to download mod: {e}");
                        None
                    }
                }
            })
            .collect()
            .await;

        Ok(processed_files)
    }

    async fn perform_modpack_import(
        &self,
        source_url: Option<&str>,
        source_path: Option<PathBuf>,
    ) -> Result<(String, Vec<ContentFile>), InstanceError> {
        let mrpack_path = match (source_url, source_path) {
            (Some(url), _) => {
                let temp_path =
                    std::env::temp_dir().join(format!("import_{}.mrpack", uuid::Uuid::new_v4()));
                self.fetch_to_disk(url, &temp_path).await?;
                temp_path
            }
            (None, Some(path)) => path,
            _ => {
                return Err(InstanceError::ContentDownloadError(
                    "No import source provided".into(),
                ))
            }
        };

        let manifest = Self::read_modpack_manifest(&mrpack_path)?;

        let (loader_type, loader_version) = super::resolve_loader_from_manifest(&manifest);

        let pack_info =
            source_url
                .and_then(|url| Self::parse_file_url(url))
                .map(|(project_id, version_id)| PackInfo {
                    plugin_id: "modrinth".to_string(),
                    modpack_id: project_id,
                    version: version_id,
                });

        let new_instance = NewInstance {
            name: manifest.name.clone(),
            game_version: manifest.dependencies.minecraft.clone(),
            mod_loader: loader_type,
            loader_version: loader_version.map(LoaderVersionPreference::Exact),
            icon_path: None,
            skip_install_instance: None,
            pack_info,
        };

        let instance_id = crate::api::instance::create(new_instance)
            .await
            .map_err(|_| InstanceError::ImportFailed {
                plugin_id: ModrinthContentProvider::ID.to_owned(),
                capability_id: self.capability.id.clone(),
            })?;

        let processed_files = self
            .deploy_modpack_files(&mrpack_path, &instance_id, manifest)
            .await?;

        if source_url.is_some() {
            let _ = std::fs::remove_file(&mrpack_path);
        }

        Ok((instance_id, processed_files))
    }
}

#[async_trait]
impl<RC: RequestClient> ContentProvider for ModrinthContentProvider<RC> {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.capability
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
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        Ok(response.into_content_search(provider_id))
    }

    async fn install_atomic(
        &self,
        install_params: &AtomicInstallParams,
    ) -> Result<ContentFile, InstanceError> {
        let project_version = self.resolve_project_version(install_params).await?;

        let file = Self::get_project_file(&project_version, install_params)?;

        let content_path = install_params
            .content_type
            .get_relative_path(&file.filename);

        let absolute_path =
            self.get_absolute_content_path(&install_params.instance_id, &content_path);

        self.fetch_to_disk(&file.url, &absolute_path).await?;

        Ok(ContentFile::from_params(CreateContentFileParams {
            name: Some(project_version.name.clone()),
            file_name: file.filename.clone(),
            size: file.size as u64,
            sha1: file.hashes.sha1.clone(),
            content_path,
            content_id: install_params.content_id.clone(),
            content_version: project_version.id,
            content_type: install_params.content_type,
            provider_id: self.get_provider_id(),
        }))
    }

    async fn install_modpack(
        &self,
        install_params: &ModpackInstallParams,
    ) -> Result<(String, Vec<ContentFile>), InstanceError> {
        let version = self.resolve_modpack_version(install_params).await?;

        let file = get_first_file_from_project_version(&version).ok_or_else(|| {
            InstanceError::ContentDownloadError(format!(
                "No files found for modpack version {}",
                version.id
            ))
        })?;

        if !file.filename.ends_with(".mrpack") {
            return Err(InstanceError::ContentDownloadError(format!(
                "File {} is not a modpack",
                file.filename
            )));
        }

        self.perform_modpack_import(Some(&file.url), None).await
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
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        let project_versions = self
            .api
            .get_project_versions(&project.id, &ProjectVersionsRequest::without_changelog())
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        let mut compatibility_map = HashMap::new();

        for instance in instances {
            let is_compatible = project_versions
                .iter()
                .any(|version| is_version_compatible(version, &project, instance));

            compatibility_map.insert(
                instance.id.clone(),
                ContentCompatibilityResult { is_compatible },
            );
        }

        Ok(compatibility_map)
    }
}
