use std::{path::PathBuf, sync::Arc};

use crate::{
    features::settings::LocationInfo,
    shared::{CacheId, CachePathResolver},
};

pub struct AssetsResolver {
    location_info: Arc<LocationInfo>,
}

impl AssetsResolver {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }
}

impl CachePathResolver for AssetsResolver {
    fn resolve(&self, namespace: &'static str, id: &CacheId) -> Option<PathBuf> {
        match (namespace, id) {
            (_, CacheId::Named(asset_id)) => {
                Some(self.location_info.assets_cache_dir().join(asset_id))
            }
            _ => None,
        }
    }
}
