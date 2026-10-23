use std::sync::Arc;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tracing::info;

use crate::{
    features::{
        events::{
            OptionalProgressBarExt, ProgressBarId, ProgressService, ProgressServiceExt,
            SharedDownload,
        },
        minecraft::{MinecraftDomainError, MinecraftDownloader, modded, vanilla, version_jar_name},
    },
    shared::request_client::{Request, RequestClient, RequestClientExt},
    shared::{
        cache::domain::{Cache, FileStore},
        cache::infra::InfinityCachedResource,
        io::domain::IoError,
        keyed_locks::KeyedLocks,
    },
};

use super::{
    AssetsService, ClientService, LibrariesService, LoggingService, VersionProgress,
    assets_service::PendingAsset, version_info_key,
};

/// The files of a game version are one shared download: one install of a version downloads
/// them at a time (a lock per version), under a shared bar, and the others wait on it.
pub struct MinecraftDownloadService<RC: RequestClient, PS: ProgressService, C: Cache, FS: FileStore>
{
    client_service: ClientService<RC, FS>,
    assets_service: AssetsService<RC, C>,
    libraries_service: LibrariesService<RC>,
    logging_service: LoggingService<RC>,
    request_client: Arc<RC>,
    progress_service: Arc<PS>,
    cached_resource: InfinityCachedResource<C>,
    version_locks: KeyedLocks<String>,
}

impl<RC: RequestClient, PS: ProgressService, C: Cache, FS: FileStore>
    MinecraftDownloadService<RC, PS, C, FS>
{
    pub fn new(
        client_service: ClientService<RC, FS>,
        assets_service: AssetsService<RC, C>,
        libraries_service: LibrariesService<RC>,
        logging_service: LoggingService<RC>,
        request_client: Arc<RC>,
        progress_service: Arc<PS>,
        cache: Arc<C>,
    ) -> Self {
        Self {
            client_service,
            assets_service,
            libraries_service,
            logging_service,
            request_client,
            progress_service,
            cached_resource: InfinityCachedResource::new(cache),
            version_locks: KeyedLocks::default(),
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
        version: &vanilla::Version,
        loader: Option<&modded::LoaderVersion>,
    ) -> Result<vanilla::VersionInfo, MinecraftDomainError> {
        let mut version_info: daedalus::minecraft::VersionInfo =
            self.fetch_json(&version.url).await?;

        if let Some(loader) = loader {
            let modded_info: daedalus::modded::PartialVersionInfo =
                self.fetch_json(&loader.url).await?;
            version_info = daedalus::modded::merge_partial_version(modded_info, version_info);
        }

        version_info.id.clone_from(version_id);

        Ok(version_info.into())
    }

    /// Download what is missing of the version, under the version's lock.
    async fn download_missing(
        &self,
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
    ) -> Result<(), MinecraftDomainError> {
        let with_legacy = version_info.assets == "legacy";
        let assets_index = self
            .assets_service
            .get_assets_index(version_info, force)
            .await?;

        let client = self.client_service.pending(version_info, force).await?;
        let libraries = self
            .libraries_service
            .pending(version_info, java_arch, force, minecraft_updated)
            .await?;
        let assets = self
            .assets_service
            .pending(&assets_index, with_legacy, force)
            .await?;
        let log_config = self.logging_service.pending(version_info, force).await;
        // Nothing to wait on: no bar, so no install gets a mirror.
        if client.is_none() && libraries.is_empty() && assets.is_empty() && log_config.is_none() {
            info!("Minecraft {} is downloaded already", version_info.id);
            return Ok(());
        }

        info!("Downloading minecraft {}", version_info.id);
        let size = client.unwrap_or(0)
            + libraries.iter().map(|library| library.size).sum::<u64>()
            + assets.iter().map(PendingAsset::size).sum::<u64>()
            + log_config.map_or(0, |file| u64::from(file.size));
        #[allow(clippy::cast_precision_loss)] // a progress total
        let bar = self
            .progress_service
            .init_shared_progress_safe(
                SharedDownload::MinecraftVersion {
                    version: version_info.id.clone(),
                },
                size.max(1) as f64,
                "Downloading Minecraft".to_owned(),
            )
            .await;
        let progress = VersionProgress::new(self.progress_service.as_ref(), bar.as_ref());

        let result = tokio::try_join! {
            async {
                if client.is_some() {
                    self.client_service
                        .download_client(version_info, force, &progress)
                        .await?;
                }
                Ok(())
            },
            self.assets_service.download_assets(&assets, &version_info.id, with_legacy, force, &progress),
            self.libraries_service.download_libraries(&libraries, version_info, java_arch, force, &progress),
            async {
                if let Some(file) = log_config {
                    self.logging_service.download_log_config(file, &progress).await?;
                }
                Ok(())
            }
        };

        bar.fail_on_err(&result).await;
        result?;

        info!("Minecraft {} downloaded", version_info.id);
        Ok(())
    }
}

#[async_trait]
impl<RC: RequestClient, PS: ProgressService, C: Cache, FS: FileStore> MinecraftDownloader
    for MinecraftDownloadService<RC, PS, C, FS>
{
    async fn download_minecraft(
        &self,
        version_info: &vanilla::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
        waiting_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        // Before the lock: a download another install runs under it is mirrored too.
        let _wait = self
            .progress_service
            .wait_on_shared(
                waiting_bar,
                SharedDownload::MinecraftVersion {
                    version: version_info.id.clone(),
                },
            )
            .await;
        let _lock = self.version_locks.lock(&version_info.id).await;

        self.download_missing(version_info, java_arch, force, minecraft_updated)
            .await
    }

    async fn get_version_info(
        &self,
        version: &vanilla::Version,
        loader: Option<&modded::LoaderVersion>,
        force: Option<bool>,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<vanilla::VersionInfo, MinecraftDomainError> {
        let force = force.unwrap_or(false);

        let version_id = get_version_id(version, loader);

        let version_info = self
            .cached_resource
            .get_cached(
                || version_info_key(version_id.clone()),
                self.fetch_version_info(&version_id, version, loader),
                || format!("version info {version_id}"),
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

fn get_version_id(version: &vanilla::Version, loader: Option<&modded::LoaderVersion>) -> String {
    version_jar_name(&version.id, loader.map(|l| l.id.as_str()))
}
