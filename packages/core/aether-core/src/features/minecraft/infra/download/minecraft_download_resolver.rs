use crate::{
    features::settings::LocationInfo,
    shared::{CacheId, CachePathResolver},
};
use std::{path::PathBuf, sync::Arc};

use super::MinecraftDownloadCacheNamespaces;

pub struct MinecraftDownloadResolver {
    location_info: Arc<LocationInfo>,
}

impl MinecraftDownloadResolver {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }
}

impl CachePathResolver for MinecraftDownloadResolver {
    fn resolve(&self, namespace: &'static str, id: &CacheId) -> Option<PathBuf> {
        match (namespace, id) {
            (ns, CacheId::Named(assets_id))
                if ns == MinecraftDownloadCacheNamespaces::AssetsIndex.as_str() =>
            {
                Some(
                    self.location_info
                        .assets_index_dir()
                        .join(format!("{assets_id}.json")),
                )
            }

            (ns, CacheId::Named(version_id))
                if ns == MinecraftDownloadCacheNamespaces::VersionInfo.as_str() =>
            {
                Some(
                    self.location_info
                        .version_dir(version_id)
                        .join(format!("{version_id}.json")),
                )
            }

            (ns, CacheId::Named(version_id))
                if ns == MinecraftDownloadCacheNamespaces::VersionJar.as_str() =>
            {
                Some(
                    self.location_info
                        .version_dir(version_id)
                        .join(format!("{version_id}.jar")),
                )
            }

            _ => None,
        }
    }
}
