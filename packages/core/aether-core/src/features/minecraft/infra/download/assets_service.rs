use std::{path::PathBuf, sync::Arc};

use bytes::Bytes;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use tracing::trace;

use crate::{
    features::{
        events::{
            utils::{try_for_each_concurrent_with_progress, ProgressConfigWithMessage},
            ProgressBarId, ProgressConfig, ProgressService, ProgressServiceExt,
        },
        minecraft::MinecraftDomainError,
        settings::LocationInfo,
    },
    libs::request_client::{Request, RequestClient, RequestClientExt},
    shared::{write_async, Cache, InfinityCachedResource, IoError},
};

use super::assets_index_key;

const MINECRAFT_RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net/";
const HASH_PREFIX_LENGTH: usize = 2;

pub struct AssetsService<RC: RequestClient, PS: ProgressService, C: Cache> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
    location_info: Arc<LocationInfo>,
    cached_resource: InfinityCachedResource<C>,
}

impl<RC: RequestClient, PS: ProgressService, C: Cache> AssetsService<RC, PS, C> {
    pub fn new(
        progress_service: Arc<PS>,
        request_client: Arc<RC>,
        location_info: Arc<LocationInfo>,
        cache: Arc<C>,
    ) -> Self {
        Self {
            progress_service,
            request_client,
            location_info,
            cached_resource: InfinityCachedResource::new(cache),
        }
    }

    fn get_asset_path(&self, hash: &str) -> PathBuf {
        self.location_info.object_dir(hash)
    }

    fn get_legacy_asset_path(&self, name: &str) -> PathBuf {
        self.location_info
            .legacy_assets_dir()
            .join(name.replace('/', &String::from(std::path::MAIN_SEPARATOR)))
    }

    fn build_asset_url(hash: &str) -> String {
        format!(
            "{MINECRAFT_RESOURCES_BASE_URL}{sub_hash}/{hash}",
            sub_hash = &hash[..HASH_PREFIX_LENGTH]
        )
    }

    async fn fetch_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, IoError> {
        self.request_client
            .fetch_json(Request::get(url))
            .await
            .map_err(get_network_error)
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Bytes, IoError> {
        self.request_client
            .fetch_bytes(Request::get(url))
            .await
            .map_err(get_network_error)
    }

    async fn fetch_assets_index(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
    ) -> Result<daedalus::minecraft::AssetsIndex, MinecraftDomainError> {
        Ok(self.fetch_json(&version_info.asset_index.url).await?)
    }

    pub async fn get_assets_index(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        force: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<daedalus::minecraft::AssetsIndex, MinecraftDomainError> {
        let assets_index = self
            .cached_resource
            .get_cached(
                || assets_index_key(version_info.asset_index.id.to_string()),
                self.fetch_assets_index(version_info),
                || format!("assets index {}", version_info.asset_index.id),
                force,
            )
            .await?;

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress_safe(loading_bar, 5.0, None)
                .await;
        }

        Ok(assets_index)
    }

    pub async fn download_assets(
        &self,
        index: &daedalus::minecraft::AssetsIndex,
        with_legacy: bool,
        force: bool,
        progress_config: Option<&ProgressConfig<'_>>,
    ) -> Result<(), MinecraftDomainError> {
        let progress_config = progress_config.map(|config| ProgressConfigWithMessage {
            progress_bar_id: config.progress_bar_id,
            total_progress: config.total_progress,
            progress_message: None,
        });

        let assets_stream = futures::stream::iter(index.objects.iter())
            .map(Ok::<(&String, &daedalus::minecraft::Asset), MinecraftDomainError>);

        let futures_count = index.objects.len();

        try_for_each_concurrent_with_progress(
            self.progress_service.clone(),
            assets_stream,
            None,
            futures_count,
            progress_config.as_ref(),
            |(name, asset)| async move {
                self.download_asset(name, asset, with_legacy, force).await
            },
        )
        .await?;

        Ok(())
    }

    pub async fn download_asset(
        &self,
        name: &str,
        asset: &daedalus::minecraft::Asset,
        with_legacy: bool,
        force: bool,
    ) -> Result<(), MinecraftDomainError> {
        let hash = &asset.hash;
        let url = Self::build_asset_url(hash);

        let asset_path = self.get_asset_path(hash);
        let legacy_path = self.get_legacy_asset_path(name);

        let need_download_asset = force || !asset_path.exists();
        let need_download_legacy = with_legacy && (force || !legacy_path.exists());

        if !need_download_asset && !need_download_legacy {
            trace!("Asset \"{}\" already downloaded", name);
            return Ok(());
        }

        trace!("Downloading asset \"{}\"\n\tfrom {}", name, url);
        let asset_data = self.fetch_bytes(&url).await?;

        tokio::try_join!(
            async {
                if need_download_asset {
                    return Ok(write_async(&asset_path, &asset_data).await?);
                }

                Ok::<(), MinecraftDomainError>(())
            },
            async {
                if need_download_legacy {
                    return Ok(write_async(&legacy_path, &asset_data).await?);
                }

                Ok::<(), MinecraftDomainError>(())
            }
        )?;

        Ok(())
    }
}

fn get_network_error<E>(error: E) -> IoError
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    IoError::IoError(std::io::Error::new(
        std::io::ErrorKind::NetworkUnreachable,
        error,
    ))
}
