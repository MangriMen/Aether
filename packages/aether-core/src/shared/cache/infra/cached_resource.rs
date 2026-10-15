use std::time::Duration;

use tracing::warn;

use crate::shared::cache::domain::{Cache, CacheKey, CachedValue};

/// How long a row outlives its `ttl` before the cache drops it.
///
/// Freshness is judged here, by [`CachedValue::updated_at`] against `ttl`; the row is kept far
/// longer, since the cache hides an expired row and a stale value must stay readable to stand in
/// for a fetch that fails.
const STALE_ROW_LIFETIME: Duration = Duration::from_hours(90 * 24);

/// A value fetched once and then read from the cache until it is older than its `ttl`. A stale
/// value is fetched again, and returned still when that fetch fails.
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

        if let Some(cached) = &cached_opt
            && !cached.is_expired(ttl)
        {
            return Ok(cached.value.clone());
        }

        match fetch_fn.await {
            Ok(value) => {
                let row_ttl = ttl.saturating_add(STALE_ROW_LIFETIME);
                self.cache
                    .set(&key, &CachedValue::new(value.clone()), row_ttl)
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

#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, pool::PoolOptions};

    use crate::{
        core::migrations::run_migrations,
        shared::cache::{domain::CacheId, infra::SqliteCache},
    };

    use super::*;

    async fn create_resource() -> CachedResource<SqliteCache> {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);
        let pool: SqlitePool = PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to create in-memory SQLite pool");
        run_migrations(&pool).await;

        CachedResource::new(SqliteCache::new(pool))
    }

    fn key() -> CacheKey<CachedValue<String>> {
        CacheKey::new("test", CacheId::Static("resource"))
    }

    async fn get(
        resource: &CachedResource<SqliteCache>,
        fetched: Result<&str, &str>,
        ttl: Duration,
    ) -> Result<String, String> {
        resource
            .get_cached(
                key,
                async { fetched.map(str::to_owned).map_err(str::to_owned) },
                || "resource".to_owned(),
                ttl,
            )
            .await
    }

    #[tokio::test]
    async fn should_return_the_stale_value_when_the_fetch_fails() {
        let resource = create_resource().await;
        let ttl = Duration::from_millis(1);
        assert_eq!(get(&resource, Ok("old"), ttl).await.unwrap(), "old");
        tokio::time::sleep(Duration::from_millis(20)).await;

        let result = get(&resource, Err("offline"), ttl).await;

        assert_eq!(result.unwrap(), "old");
    }

    #[tokio::test]
    async fn should_fetch_again_when_the_value_is_stale() {
        let resource = create_resource().await;
        let ttl = Duration::from_millis(1);
        get(&resource, Ok("old"), ttl).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;

        assert_eq!(get(&resource, Ok("new"), ttl).await.unwrap(), "new");
        assert_eq!(
            get(&resource, Err("offline"), Duration::from_hours(1))
                .await
                .unwrap(),
            "new"
        );
    }
}
