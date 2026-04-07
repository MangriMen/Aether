use crate::{
    features::settings::LocationInfo,
    shared::{CacheId, CachePathResolver},
};
use std::{path::PathBuf, sync::Arc};

use super::MinecraftMetadataCacheNamespaces;

pub struct MinecraftMetadataResolver {
    location_info: Arc<LocationInfo>,
}

impl MinecraftMetadataResolver {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn root(&self) -> PathBuf {
        self.location_info.cache_dir().join("minecraft")
    }
}

impl CachePathResolver for MinecraftMetadataResolver {
    fn resolve(&self, namespace: &'static str, id: &CacheId) -> Option<PathBuf> {
        let root = self.root();

        match (namespace, id) {
            (ns, CacheId::Static("version-manifest"))
                if ns == MinecraftMetadataCacheNamespaces::VersionManifest.as_str() =>
            {
                Some(root.join("version-manifest.json"))
            }

            (ns, CacheId::Named(loader))
                if ns == MinecraftMetadataCacheNamespaces::LoaderManifest.as_str() =>
            {
                Some(root.join(format!("{loader}-version-manifest.json")))
            }

            _ => None,
        }
    }
}
