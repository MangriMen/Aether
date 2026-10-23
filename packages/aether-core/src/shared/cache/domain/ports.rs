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

    /// Delete every value past its expiry, and say how many went.
    ///
    /// [`Cache::get`] already hides such values; this frees what they take.
    async fn purge_expired(&self) -> u64;

    /// The values of `keys`, in their order: `None` where [`Cache::get`] would give none.
    ///
    /// By default one [`Cache::get`] per key; a store that can read many at once does.
    async fn get_many<T: DeserializeOwned + Send + Sync>(
        &self,
        keys: &[CacheKey<T>],
    ) -> Vec<Option<T>> {
        futures::future::join_all(keys.iter().map(|key| self.get(key))).await
    }

    /// [`Cache::set`] each value under its key, for `ttl`. Whether they were all written, for
    /// a caller that would otherwise ask for them again and again.
    ///
    /// By default one [`Cache::set`] per entry, which cannot tell, and says `true`; a store
    /// that can write many at once does, and tells.
    async fn set_many<T: Serialize + Send + Sync>(
        &self,
        entries: &[(CacheKey<T>, T)],
        ttl: Duration,
    ) -> bool {
        futures::future::join_all(entries.iter().map(|(key, value)| self.set(key, value, ttl)))
            .await;
        true
    }
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
    /// Like [`Self::invalidate`], but says why a file that exists could not be deleted.
    async fn delete(&self, key: &CacheKey<()>) -> std::io::Result<()>;
}

// --- AssetsStorage ---

#[async_trait]
pub trait AssetsStorage: Send + Sync {
    async fn import_file(&self, source: &Path) -> Result<String, AssetError>;

    /// Deletes the asset `id` as [`Self::import_file`] returned it. Assets are shared by
    /// content, so the caller makes sure nothing uses it. Gone already is a no-op, and so is a
    /// name that is not a plain asset name; a file that cannot be deleted is an error.
    async fn remove(&self, id: &str) -> Result<(), AssetError>;
}
