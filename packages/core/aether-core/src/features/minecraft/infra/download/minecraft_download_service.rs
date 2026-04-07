use std::sync::Arc;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tracing::info;

use crate::{
    features::{
        events::{ProgressBarId, ProgressConfig, ProgressService, ProgressServiceExt},
        minecraft::{MinecraftDomainError, MinecraftDownloader},
    },
    libs::request_client::{Request, RequestClient, RequestClientExt},
    shared::{Cache, FileStore, InfinityCachedResource, IoError},
};

use super::{version_info_key, AssetsService, ClientService, LibrariesService};

pub struct MinecraftDownloadService<RC: RequestClient, PS: ProgressService, C: Cache, FS: FileStore>
{
    client_service: ClientService<RC, PS, FS>,
    assets_service: AssetsService<RC, PS, C>,
    libraries_service: LibrariesService<RC, PS>,
    request_client: Arc<RC>,
    progress_service: Arc<PS>,
    cached_resource: InfinityCachedResource<C>,
}

impl<RC: RequestClient, PS: ProgressService, C: Cache, FS: FileStore>
    MinecraftDownloadService<RC, PS, C, FS>
{
    pub fn new(
        client_service: ClientService<RC, PS, FS>,
        assets_service: AssetsService<RC, PS, C>,
        libraries_service: LibrariesService<RC, PS>,
        request_client: Arc<RC>,
        progress_service: Arc<PS>,
        cache: Arc<C>,
    ) -> Self {
        Self {
            client_service,
            assets_service,
            libraries_service,
            request_client,
            progress_service,
            cached_resource: InfinityCachedResource::new(cache),
        }
    }

    async fn fetch_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, IoError> {
        self.request_client
            .fetch_json(Request::get(url))
            .await
            .map_err(|err| {
                IoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                ))
            })
    }

    async fn fetch_version_info(
        &self,
        version_id: &String,
        version: &daedalus::minecraft::Version,
        loader: Option<&daedalus::modded::LoaderVersion>,
    ) -> Result<daedalus::minecraft::VersionInfo, MinecraftDomainError> {
        let mut version_info = self.fetch_json(&version.url).await?;

        if let Some(loader) = loader {
            let modded_info = self.fetch_json(&loader.url).await?;
            version_info = daedalus::modded::merge_partial_version(modded_info, version_info);
        }

        version_info.id.clone_from(version_id);

        Ok(version_info)
    }
}

#[async_trait]
impl<RC: RequestClient, PS: ProgressService, C: Cache, FS: FileStore> MinecraftDownloader
    for MinecraftDownloadService<RC, PS, C, FS>
{
    async fn download_minecraft(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        info!("Downloading minecraft {}", version_info.id);

        let assets_index = self
            .assets_service
            .get_assets_index(version_info, force, progress_bar_id)
            .await?;

        let start_progress = if version_info
            .processors
            .as_ref()
            .is_some_and(|p| !p.is_empty())
        {
            25.0
        } else {
            40.0
        };
        let progress_config = progress_bar_id.map(|progress_bar_id| ProgressConfig {
            progress_bar_id,
            total_progress: start_progress,
        });

        tokio::try_join! {
            self.client_service.download_client(version_info, force, progress_bar_id),
            self.assets_service.download_assets(&assets_index, version_info.assets == "legacy", force, progress_config.as_ref()),
            self.libraries_service.download_libraries( version_info.libraries.as_slice(), version_info, java_arch, force, minecraft_updated, progress_config.as_ref())
        }?;

        info!("Minecraft {} downloaded", version_info.id);

        Ok(())
    }

    async fn get_version_info(
        &self,
        version: &daedalus::minecraft::Version,
        loader: Option<&daedalus::modded::LoaderVersion>,
        force: Option<bool>,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<daedalus::minecraft::VersionInfo, MinecraftDomainError> {
        let force = force.unwrap_or(false);

        let version_id = get_version_id(version, loader);

        let version_info = self
            .cached_resource
            .get_cached(
                || version_info_key(version_id.to_string()),
                self.fetch_version_info(&version_id, version, loader),
                || format!("version info {}", version_id),
                force,
            )
            .await?;

        if let Some(bar) = progress_bar_id {
            self.progress_service
                .emit_progress_safe(bar, 5.0, None)
                .await;
        }

        Ok(version_info)
    }
}

fn get_version_id(
    version: &daedalus::minecraft::Version,
    loader: Option<&daedalus::modded::LoaderVersion>,
) -> String {
    loader.map_or_else(
        || version.id.clone(),
        |l| format!("{}-{}", version.id, l.id),
    )
}
