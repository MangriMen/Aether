use std::{path::PathBuf, time::Duration};

use async_trait::async_trait;
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use tracing::debug;

use crate::shared::{
    create_dir_all, read_async, read_json_async, remove_file, write_json_async, Cache, CacheKey,
    CachePathResolver, FileStore,
};

pub struct FileCache<R> {
    resolver: R,
}

impl<R: CachePathResolver> FileCache<R> {
    pub fn new(resolver: R) -> Self {
        Self { resolver }
    }

    fn path<T>(&self, key: &CacheKey<T>) -> Option<PathBuf> {
        self.resolver.resolve(key.namespace, key.id())
    }
}

#[async_trait]
impl<R: CachePathResolver> Cache for FileCache<R> {
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &CacheKey<T>) -> Option<T> {
        let path = self.path(key)?;

        read_json_async(path).await.unwrap_or(None)
    }

    async fn set<T: Serialize + Send + Sync>(&self, key: &CacheKey<T>, value: &T, _ttl: Duration) {
        let Some(path) = self.path(key) else {
            return;
        };

        if let Some(parent) = path.parent() {
            if let Err(err) = create_dir_all(&parent).await {
                debug!(
                    "Failed to create cache dir {:?}. {:?}",
                    path.to_path_buf(),
                    err
                );
            }
        }

        if let Err(err) = write_json_async(&path, value).await {
            debug!(
                "Failed to write cached value to {:?}. {:?}",
                path.to_path_buf(),
                err
            );
        }
    }

    async fn exists<T: Send + Sync>(&self, key: &CacheKey<T>) -> bool {
        let Some(path) = self.path(key) else {
            return false;
        };

        path.exists()
    }

    async fn invalidate<T: Send + Sync>(&self, key: &CacheKey<T>) {
        let Some(path) = self.path(key) else {
            return;
        };

        if let Err(err) = remove_file(&path).await {
            debug!("Failed to remove cache file {:?}. {:?}", path, err)
        }
    }
}

#[async_trait]
impl<R: CachePathResolver> FileStore for FileCache<R> {
    async fn exists(&self, key: &CacheKey<()>) -> bool {
        self.resolver
            .resolve(key.namespace, key.id())
            .is_some_and(|p| p.exists())
    }

    async fn read(&self, key: &CacheKey<()>) -> Option<Bytes> {
        let path = self.path(key)?;
        read_async(path).await.ok().map(Into::into)
    }

    async fn write(&self, key: &CacheKey<()>, data: Bytes) {
        let Some(path) = self.path(key) else {
            return;
        };

        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent).await;
        }

        let _ = tokio::fs::write(path, data).await;
    }

    async fn invalidate(&self, key: &CacheKey<()>) {
        let Some(path) = self.path(key) else {
            return;
        };

        if let Err(err) = remove_file(&path).await {
            debug!("Failed to remove cache file {:?}. {:?}", path, err)
        }
    }
}
