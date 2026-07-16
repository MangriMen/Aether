use std::{
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
            CapabilityMetadata, ContentFile, ContentType, CreateContentFileParams, DownloadContext,
            InstanceError, PackInfo, PackInstallParams, PackManager, PackManagerCapabilityMetadata,
            PackMetadata, PackSource, ProviderId,
            infra::content_providers::modrinth::{
                api_client::{
                    MODRINTH_API_URL, ModrinthApiClient, ModrinthIndex, ModrinthIndexFile,
                    ProjectVersionResponse, ProjectVersionsRequest,
                },
                content_provider::{
                    get_first_file_from_project_version, resolve_loader_from_manifest,
                },
            },
        },
        settings::LocationInfo,
    },
    shared::io::infra::{create_dir_all, write_async},
    shared::request_client::{Request, RequestClient},
};

pub struct ModrinthPackManager<RC> {
    location_info: Arc<LocationInfo>,
    request_client: Arc<RC>,
    api: ModrinthApiClient<RC>,
    metadata: PackManagerCapabilityMetadata,
}

impl ModrinthPackManager<()> {
    pub const ID: &'static str = "core:modrinth";
}

impl<RC: RequestClient> ModrinthPackManager<RC> {
    pub fn new(
        location_info: Arc<LocationInfo>,
        base_headers: Option<reqwest::header::HeaderMap>,
        request_client: Arc<RC>,
    ) -> Self {
        let metadata = PackManagerCapabilityMetadata {
            base: CapabilityMetadata {
                id: "modrinth-pack-manager".to_string(),
                name: "Modrinth".to_string(),
                description: Some("Installs modpacks from Modrinth (.mrpack)".to_string()),
                icon: None,
            },
            supports_install: true,
            supports_update: false,
            supports_check_updates: false,
            field_label: Some("Modrinth pack URL or ID".to_string()),
            supported_extensions: vec!["mrpack".to_string()],
        };

        Self {
            request_client: request_client.clone(),
            api: ModrinthApiClient::new(MODRINTH_API_URL.to_string(), base_headers, request_client),
            location_info,
            metadata,
        }
    }

    async fn fetch_to_disk(&self, url: &str, path: &Path) -> Result<(), InstanceError> {
        let file_bytes = self
            .request_client
            .fetch_bytes(Request::get(url))
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        write_async(path, &file_bytes)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }

    async fn resolve_modpack_version(
        &self,
        pack_id: &str,
        version_id: Option<&str>,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        if let Some(version_id) = version_id {
            self.api
                .get_project_version(version_id)
                .await
                .map_err(|err| InstanceError::ContentDownloadError(err.clone()))
        } else {
            let versions = self
                .api
                .get_project_versions(pack_id, &ProjectVersionsRequest::without_changelog())
                .await
                .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

            versions
                .first()
                .cloned()
                .ok_or(InstanceError::ContentDownloadError(
                    "No versions found for modpack".to_string(),
                ))
        }
    }

    fn read_modpack_manifest(mrpack_path: &Path) -> Result<ModrinthIndex, InstanceError> {
        const INDEX_PATH: &str = "modrinth.index.json";

        let file =
            std::fs::File::open(mrpack_path).map_err(|e| InstanceError::ContentProviderError {
                reason: format!("Failed to open .mrpack file: {e}"),
            })?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| InstanceError::ContentProviderError {
                reason: format!("Invalid .mrpack archive: {e}"),
            })?;

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
        const MODPACK_OVERRIDES_DIR: &str = "overrides/";

        let file =
            std::fs::File::open(mrpack_path).map_err(|e| InstanceError::ContentProviderError {
                reason: format!("Failed to open pack: {e}"),
            })?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| InstanceError::ContentProviderError {
                reason: format!("Invalid zip: {e}"),
            })?;

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
                        reason: format!("Failed to create directory {}: {e}", full_path.display()),
                    }
                })?;
                continue;
            }

            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    InstanceError::ContentProviderError {
                        reason: format!(
                            "Failed to create parent directory for {}: {e}",
                            full_path.display()
                        ),
                    }
                })?;
            }

            let mut outfile = std::fs::File::create(&full_path).map_err(|e| {
                InstanceError::ContentProviderError {
                    reason: format!("Failed to create file {}: {e}", full_path.display()),
                }
            })?;

            std::io::copy(&mut zip_file, &mut outfile).map_err(|e| {
                InstanceError::ContentProviderError {
                    reason: format!("Failed to copy data to {}: {e}", full_path.display()),
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
        let Some(url) = mod_file.downloads.first() else {
            return Ok(None);
        };

        let Some((project_id, version_id)) = Self::parse_file_url(url) else {
            return Ok(None);
        };

        let content_path = PathBuf::from(&mod_file.path);
        let Some(content_type) = ContentType::get_from_parent_folder(&content_path) else {
            return Ok(None);
        };

        let target_path = instance_dir.join(&content_path);

        if let Some(parent) = target_path.parent() {
            create_dir_all(parent)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
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
            provider_id: ProviderId {
                plugin_id: ModrinthPackManager::ID.to_owned(),
                capability_id: "modrinth-pack-manager".to_string(),
            },
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
}

#[async_trait]
impl<RC: RequestClient> PackManager for ModrinthPackManager<RC> {
    fn metadata(&self) -> &PackManagerCapabilityMetadata {
        &self.metadata
    }

    async fn resolve_pack_metadata(
        &self,
        params: &PackInstallParams,
    ) -> Result<PackMetadata, InstanceError> {
        let (pack_id, version_id) = match &params.source {
            PackSource::Registry {
                provider_id: _,
                pack_id,
                version_id,
            } => (pack_id.clone(), Some(version_id.clone())),
            _ => {
                return Err(InstanceError::UnsupportedOperation(
                    "ModrinthPackManager requires a Registry source".into(),
                ));
            }
        };

        let version = self
            .resolve_modpack_version(&pack_id, version_id.as_deref())
            .await?;

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

        let mrpack_path =
            std::env::temp_dir().join(format!("resolve_{}.mrpack", uuid::Uuid::new_v4()));
        self.fetch_to_disk(&file.url, &mrpack_path).await?;

        let manifest = Self::read_modpack_manifest(&mrpack_path)?;

        let _ = std::fs::remove_file(&mrpack_path);

        let (loader_type, loader_version) = resolve_loader_from_manifest(&manifest);

        let pack_info =
            Self::parse_file_url(&file.url).map(|(resolved_project_id, resolved_version_id)| {
                PackInfo {
                    provider_id: ProviderId {
                        plugin_id: ModrinthPackManager::ID.to_owned(),
                        capability_id: self.metadata.base.id.clone(),
                    },
                    modpack_id: resolved_project_id,
                    version_id: resolved_version_id,
                }
            });

        Ok(PackMetadata {
            name: manifest.name.clone(),
            game_version: manifest.dependencies.minecraft.clone(),
            mod_loader: loader_type,
            loader_version,
            pack_info,
        })
    }

    async fn install(
        &self,
        instance_id: &str,
        params: &PackInstallParams,
        _ctx: &DownloadContext,
    ) -> Result<(), InstanceError> {
        let (pack_id, version_id) = match &params.source {
            PackSource::Registry {
                provider_id: _,
                pack_id,
                version_id,
            } => (pack_id.clone(), Some(version_id.clone())),
            _ => {
                return Err(InstanceError::UnsupportedOperation(
                    "ModrinthPackManager requires a Registry source".into(),
                ));
            }
        };

        let version = self
            .resolve_modpack_version(&pack_id, version_id.as_deref())
            .await?;

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

        let mrpack_path =
            std::env::temp_dir().join(format!("install_{}.mrpack", uuid::Uuid::new_v4()));
        self.fetch_to_disk(&file.url, &mrpack_path).await?;

        let manifest = Self::read_modpack_manifest(&mrpack_path)?;

        self.deploy_modpack_files(&mrpack_path, instance_id, manifest)
            .await?;

        let _ = std::fs::remove_file(&mrpack_path);

        Ok(())
    }
}
