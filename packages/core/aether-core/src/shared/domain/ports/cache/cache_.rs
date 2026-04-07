use std::time::Duration;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::shared::CacheKey;

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &CacheKey<T>) -> Option<T>;
    async fn set<T: Serialize + Send + Sync>(&self, key: &CacheKey<T>, value: &T, ttl: Duration);
    async fn exists<T: Send + Sync>(&self, key: &CacheKey<T>) -> bool;
    async fn invalidate<T: Send + Sync>(&self, key: &CacheKey<T>);
}
