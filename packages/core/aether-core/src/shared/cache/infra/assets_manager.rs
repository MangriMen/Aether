use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;

use crate::shared::{
    cache::domain::{AssetError, AssetsStorage, CacheId, CacheKey, FileStore},
    hash::infra::sha1_async,
    io::infra::read_async,
};

pub const ASSETS_CACHE_NAMESPACE: &str = "assets";

pub struct FsAssetsStorage<C> {
    cache: Arc<C>,
}

impl<C: FileStore> FsAssetsStorage<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl<C: FileStore> AssetsStorage for FsAssetsStorage<C> {
    async fn import_file(&self, source: impl AsRef<Path> + Send) -> Result<String, AssetError> {
        let source = source.as_ref();

        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");

        let data = read_async(source)
            .await
            .map_err(|err| AssetError::ReadSource(err.to_string()))?;
        let bytes = Bytes::from_owner(data);

        let hash = sha1_async(bytes.clone()).await;

        let filename = format!("{hash}.{ext}");
        let key = CacheKey::new(ASSETS_CACHE_NAMESPACE, CacheId::Named(filename.clone()));

        if !self.cache.exists(&key).await {
            self.cache.write(&key, bytes).await;
        }

        Ok(filename)
    }
}
