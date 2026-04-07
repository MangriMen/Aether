use std::time::Duration;

use tracing::warn;

use crate::shared::{Cache, CacheKey, CachedValue};

pub struct CachedResource<C: Cache> {
    cache: C,
}

impl<C: Cache> CachedResource<C> {
    pub fn new(cache: C) -> Self {
        Self { cache }
    }

    pub async fn get_cached<T, Fut, KeyFn, ContextFn, E>(
        &self,
        key_fn: KeyFn,
        fetch_fn: Fut,
        context_fn: ContextFn,
        ttl: Duration,
    ) -> Result<T, E>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<T, E>>,
        KeyFn: Fn() -> CacheKey<CachedValue<T>>,
        ContextFn: Fn() -> String,
    {
        let key = key_fn();
        let cached_opt = self.cache.get::<CachedValue<T>>(&key).await;

        if let Some(cached) = &cached_opt {
            if !cached.is_expired(ttl) {
                return Ok(cached.value.clone());
            }
        }

        match fetch_fn.await {
            Ok(value) => {
                self.cache
                    .set(&key, &CachedValue::new(value.clone()), ttl)
                    .await;
                Ok(value)
            }
            Err(err) => {
                if let Some(cached) = cached_opt {
                    warn!("Failed to fetch {}. Returning expired cached", context_fn());
                    return Ok(cached.value.clone());
                }
                Err(err)
            }
        }
    }
}
