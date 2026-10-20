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
    async fn import_file(&self, source: &Path) -> Result<String, AssetError> {
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");

        let data = read_async(source)
            .await
            .map_err(|err| AssetError::ReadSource(err.to_string()))?;
        let bytes = Bytes::from_owner(data);

        let hash = sha1_async(bytes.clone())
            .await
            .map_err(|err| AssetError::ReadSource(err.to_string()))?;

        let filename = format!("{hash}.{ext}");
        let key = CacheKey::new(ASSETS_CACHE_NAMESPACE, CacheId::Named(filename.clone()));

        if !self.cache.exists(&key).await {
            self.cache.write(&key, bytes).await;
        }

        Ok(filename)
    }

    async fn remove(&self, id: &str) -> Result<(), AssetError> {
        // Ids come from the database: refuse anything that could name a file elsewhere.
        if id.is_empty() || id.contains(['/', '\\']) || id.contains("..") {
            return Ok(());
        }
        let key = CacheKey::new(ASSETS_CACHE_NAMESPACE, CacheId::Named(id.to_owned()));
        self.cache
            .delete(&key)
            .await
            .map_err(|err| AssetError::Remove(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct RecordingStore {
        invalidated: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl FileStore for RecordingStore {
        async fn exists(&self, _: &CacheKey<()>) -> bool {
            false
        }
        async fn read(&self, _: &CacheKey<()>) -> Option<Bytes> {
            None
        }
        async fn write(&self, _: &CacheKey<()>, _: Bytes) {}
        async fn invalidate(&self, key: &CacheKey<()>) {
            if let CacheId::Named(name) = key.id() {
                self.invalidated.lock().unwrap().push(name.clone());
            }
        }
        async fn delete(&self, key: &CacheKey<()>) -> std::io::Result<()> {
            self.invalidate(key).await;
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_remove_only_plain_asset_names() {
        let store = Arc::new(RecordingStore::default());
        let assets = FsAssetsStorage::new(store.clone());

        for id in ["", "../x.png", "a/b.png", "a\\b.png", "..", "ok.png"] {
            assets.remove(id).await.unwrap();
        }

        assert_eq!(*store.invalidated.lock().unwrap(), ["ok.png"]);
    }
}
