use std::{path::PathBuf, sync::Arc};

use bytes::Bytes;
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use tracing::trace;

use crate::{
    features::{
        events::{ProgressEventType, ProgressService},
        minecraft::{MinecraftDomainError, vanilla},
        settings::LocationInfo,
    },
    shared::{
        cache::{domain::Cache, infra::InfinityCachedResource},
        io::{domain::IoError, infra::write_replacing_async},
        keyed_locks::KeyedLocks,
        request_client::{Request, RequestClient, RequestClientExt},
    },
};

use super::{VersionProgress, assets_index_key, checked_get};

const MINECRAFT_RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net/";
const HASH_PREFIX_LENGTH: usize = 2;

/// Assets are shared by every game version that lists them; one of them is written by one
/// install at a time ([`KeyedLocks`] by path), and whole or not at all, so another never takes
/// a file still being written for one that is there.
pub struct AssetsService<RC: RequestClient, C: Cache> {
    metadata_client: Arc<RC>,
    download_client: Arc<RC>,
    location_info: Arc<LocationInfo>,
    cached_resource: InfinityCachedResource<C>,
    path_locks: KeyedLocks<PathBuf>,
}

/// An asset not there yet, as an object or, for a legacy version, under its name.
pub(crate) struct PendingAsset<'a> {
    name: &'a str,
    asset: &'a vanilla::Asset,
}

impl PendingAsset<'_> {
    /// The asset's size, as the index says.
    pub(super) fn size(&self) -> u64 {
        self.asset.size.into()
    }
}

impl<RC: RequestClient, C: Cache> AssetsService<RC, C> {
    pub fn new(
        metadata_client: Arc<RC>,
        download_client: Arc<RC>,
        location_info: Arc<LocationInfo>,
        cache: Arc<C>,
    ) -> Self {
        Self {
            metadata_client,
            download_client,
            location_info,
            cached_resource: InfinityCachedResource::new(cache),
            path_locks: KeyedLocks::default(),
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
        self.metadata_client
            .fetch_json(Request::get(url))
            .await
            .map_err(get_network_error)
    }

    async fn fetch_bytes(&self, url: &str, sha1: &str) -> Result<Bytes, IoError> {
        self.download_client
            .fetch_bytes(checked_get(url, sha1))
            .await
            .map_err(get_network_error)
    }

    async fn fetch_assets_index(
        &self,
        version_info: &vanilla::VersionInfo,
    ) -> Result<vanilla::AssetsIndex, MinecraftDomainError> {
        Ok(self.fetch_json(&version_info.asset_index.url).await?)
    }

    pub async fn get_assets_index(
        &self,
        version_info: &vanilla::VersionInfo,
        force: bool,
    ) -> Result<vanilla::AssetsIndex, MinecraftDomainError> {
        self.cached_resource
            .get_cached(
                || assets_index_key(version_info.asset_index.id.clone()),
                self.fetch_assets_index(version_info),
                || format!("assets index {}", version_info.asset_index.id),
                force,
            )
            .await
    }

    /// The assets of `index` to download: all when `force`d, else those missing as an object
    /// or, for a legacy version, under their name. The thousands of paths are looked at on a
    /// blocking thread.
    pub(crate) async fn pending<'a>(
        &self,
        index: &'a vanilla::AssetsIndex,
        with_legacy: bool,
        force: bool,
    ) -> Result<Vec<PendingAsset<'a>>, MinecraftDomainError> {
        let assets = index
            .objects
            .iter()
            .map(|(name, asset)| PendingAsset { name, asset });
        if force {
            return Ok(assets.collect());
        }

        let paths: Vec<_> = index
            .objects
            .iter()
            .map(|(name, asset)| {
                (
                    self.get_asset_path(&asset.hash),
                    with_legacy.then(|| self.get_legacy_asset_path(name)),
                )
            })
            .collect();
        let missing = tokio::task::spawn_blocking(move || {
            paths
                .iter()
                .map(|(object, legacy)| {
                    !object.exists() || legacy.as_ref().is_some_and(|legacy| !legacy.exists())
                })
                .collect::<Vec<_>>()
        })
        .await
        .map_err(|err| IoError::from(std::io::Error::other(err)))?;

        Ok(assets
            .zip(missing)
            .filter_map(|(asset, missing)| missing.then_some(asset))
            .collect())
    }

    /// Download `assets` under one bar in `progress`, counted in files and bytes: there are
    /// thousands of them, most a few kilobytes.
    pub(crate) async fn download_assets<PS: ProgressService>(
        &self,
        assets: &[PendingAsset<'_>],
        version_id: &str,
        with_legacy: bool,
        force: bool,
        progress: &VersionProgress<'_, PS>,
    ) -> Result<(), MinecraftDomainError> {
        if assets.is_empty() {
            return Ok(());
        }
        let file_count = u32::try_from(assets.len()).unwrap_or(u32::MAX);
        let bar = progress
            .start(
                ProgressEventType::MinecraftAssetsDownload {
                    version: version_id.to_owned(),
                    file_count,
                },
                f64::from(file_count),
            )
            .await;
        progress
            .add_bytes_total(&bar, assets.iter().map(PendingAsset::size).sum())
            .await;

        let result = futures::stream::iter(assets.iter().map(Ok))
            .try_for_each_concurrent(None, |pending| async {
                let bytes = self
                    .download_asset(pending.name, pending.asset, with_legacy, force)
                    .await?;
                progress.add_step(&bar, bytes, pending.size()).await;
                Ok::<(), MinecraftDomainError>(())
            })
            .await;
        bar.settle(&result).await;
        result
    }

    /// Download one asset, unless it is there (and not `force`d): checked under the locks of
    /// its paths, as another install may have put it there meanwhile. The bytes it brings in
    /// place: its size when it was there already.
    async fn download_asset(
        &self,
        name: &str,
        asset: &vanilla::Asset,
        with_legacy: bool,
        force: bool,
    ) -> Result<u64, MinecraftDomainError> {
        let hash = &asset.hash;
        let url = Self::build_asset_url(hash);

        let asset_path = self.get_asset_path(hash);
        let legacy_path = self.get_legacy_asset_path(name);

        // Always the object first, then the name: no two installs wait on each other's.
        let _asset_lock = self.path_locks.lock(&asset_path).await;
        let _legacy_lock = if with_legacy {
            Some(self.path_locks.lock(&legacy_path).await)
        } else {
            None
        };

        let need_download_asset = force || !asset_path.exists();
        let need_download_legacy = with_legacy && (force || !legacy_path.exists());

        if !need_download_asset && !need_download_legacy {
            trace!("Asset \"{}\" already downloaded", name);
            return Ok(asset.size.into());
        }

        trace!("Downloading asset \"{}\"\n\tfrom {}", name, url);
        let asset_data = self.fetch_bytes(&url, hash).await?;
        let size = asset_data.len() as u64;

        tokio::try_join!(
            async {
                if need_download_asset {
                    write_replacing_async(&asset_path, asset_data.clone()).await?;
                }

                Ok::<(), MinecraftDomainError>(())
            },
            async {
                if need_download_legacy {
                    write_replacing_async(&legacy_path, asset_data.clone()).await?;
                }

                Ok::<(), MinecraftDomainError>(())
            }
        )?;

        Ok(size)
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
