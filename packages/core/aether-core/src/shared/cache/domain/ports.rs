use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};

use crate::shared::cache::domain::models::{AssetError, CacheId, CacheKey};

// --- Cache ---

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &CacheKey<T>) -> Option<T>;
    async fn set<T: Serialize + Send + Sync>(&self, key: &CacheKey<T>, value: &T, ttl: Duration);
    async fn exists<T: Send + Sync>(&self, key: &CacheKey<T>) -> bool;
    async fn invalidate<T: Send + Sync>(&self, key: &CacheKey<T>);
}

// --- CachePathResolver ---

pub trait CachePathResolver: Send + Sync {
    fn resolve(&self, namespace: &'static str, id: &CacheId) -> Option<PathBuf>;
}

// --- FileStore ---

#[async_trait]
pub trait FileStore: Send + Sync {
    async fn exists(&self, key: &CacheKey<()>) -> bool;
    async fn read(&self, key: &CacheKey<()>) -> Option<Bytes>;
    async fn write(&self, key: &CacheKey<()>, value: Bytes);
    async fn invalidate(&self, key: &CacheKey<()>);
}

// --- AssetsStorage ---

#[async_trait]
pub trait AssetsStorage: Send + Sync {
    async fn import_file(&self, source: impl AsRef<Path> + Send) -> Result<String, AssetError>;
}
