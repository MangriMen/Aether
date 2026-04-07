use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use tracing::debug;

use crate::shared::{Cache, CacheKey, FileStore};

pub struct InfinityCachedResource<C> {
    pub cache: Arc<C>,
}

impl<C: Cache> InfinityCachedResource<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self { cache }
    }

    pub async fn get_cached<T, Fut, KeyFn, ContextFn, E>(
        &self,
        key_fn: KeyFn,
        fetch_fn: Fut,
        context_fn: ContextFn,
        force: bool,
    ) -> Result<T, E>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<T, E>>,
        KeyFn: Fn() -> CacheKey<T>,
        ContextFn: Fn() -> String,
    {
        let key = key_fn();

        if !force {
            if let Some(value) = self.cache.get::<T>(&key).await {
                debug!("Get {} from cache", context_fn());
                return Ok(value);
            }
        }

        let value = fetch_fn.await?;
        debug!("Fetched {}", context_fn());

        self.cache.set(&key, &value, Duration::ZERO).await;

        Ok(value)
    }
}

impl<FS: FileStore> InfinityCachedResource<FS> {
    pub async fn ensure<Fut, KeyFn, ContextFn, E>(
        &self,
        key_fn: KeyFn,
        fetch_fn: Fut,
        context_fn: ContextFn,
        force: bool,
    ) -> Result<(), E>
    where
        Fut: std::future::Future<Output = Result<Bytes, E>>,
        KeyFn: Fn() -> CacheKey<()>,
        ContextFn: Fn() -> String,
    {
        let key = key_fn();

        if !force && self.cache.exists(&key).await {
            debug!("{} already downloaded", context_fn());
            return Ok(());
        }

        let value = fetch_fn.await?;
        debug!("{} fetched", context_fn());

        self.cache.write(&key, value).await;

        Ok(())
    }
}
