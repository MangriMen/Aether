use async_trait::async_trait;
use bytes::Bytes;

use crate::shared::CacheKey;

#[async_trait]
pub trait FileStore: Send + Sync {
    async fn exists(&self, key: &CacheKey<()>) -> bool;
    async fn read(&self, key: &CacheKey<()>) -> Option<Bytes>;
    async fn write(&self, key: &CacheKey<()>, value: Bytes);
    async fn invalidate(&self, key: &CacheKey<()>);
}
