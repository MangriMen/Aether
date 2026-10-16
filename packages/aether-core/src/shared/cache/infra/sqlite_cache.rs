use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Executor, QueryBuilder, Row, Sqlite, SqlitePool};
use std::{collections::HashMap, time::Duration};

use crate::shared::{
    cache::domain::{Cache, CacheKey},
    housekeeping::{CacheCleaner, CacheKind, Housekeeping},
};

/// How many keys one [`Cache::get_many`] query names: well under SQLite's limit of bound
/// variables (999 before 3.32).
const BATCH_SIZE: usize = 500;

pub struct SqliteCache {
    pool: SqlitePool,
}

impl SqliteCache {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// When a row put now for `ttl` expires.
fn expires_at(ttl: Duration) -> DateTime<Utc> {
    Utc::now() + ChronoDuration::from_std(ttl).unwrap_or_else(|_| ChronoDuration::zero())
}

/// Put `value` under `key` until `expires_at`, replacing what was there. A value that does not
/// serialize, or a failed write, is dropped. Whether it was written.
async fn upsert<'e, T: Serialize>(
    executor: impl Executor<'e, Database = Sqlite>,
    key: &CacheKey<T>,
    value: &T,
    expires_at: DateTime<Utc>,
) -> bool {
    let Ok(value_json) = serde_json::to_string(value) else {
        return false;
    };

    let namespace = key.namespace();
    let id = key.id().as_ref();

    sqlx::query!(
        "INSERT INTO universal_cache (namespace, key, value, expires_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(namespace, key) DO UPDATE SET
                value = excluded.value,
                expires_at = excluded.expires_at",
        namespace,
        id,
        value_json,
        expires_at
    )
    .execute(executor)
    .await
    .is_ok()
}

#[async_trait]
impl Cache for SqliteCache {
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &CacheKey<T>) -> Option<T> {
        let now = Utc::now();

        let row = sqlx::query(
            "SELECT value FROM universal_cache 
             WHERE namespace = ? AND key = ? AND expires_at > ?",
        )
        .bind(key.namespace())
        .bind(key.id().as_ref())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        let value_json: String = row.get(0);
        serde_json::from_str(&value_json).ok()
    }

    async fn set<T: Serialize + Send + Sync>(&self, key: &CacheKey<T>, value: &T, ttl: Duration) {
        upsert(&self.pool, key, value, expires_at(ttl)).await;
    }

    /// One query per namespace and [`BATCH_SIZE`] keys.
    async fn get_many<T: DeserializeOwned + Send + Sync>(
        &self,
        keys: &[CacheKey<T>],
    ) -> Vec<Option<T>> {
        let now = Utc::now();

        let mut ids_by_namespace: HashMap<&str, Vec<&str>> = HashMap::new();
        for key in keys {
            ids_by_namespace
                .entry(key.namespace())
                .or_default()
                .push(key.id().as_ref());
        }

        let mut found: HashMap<&str, HashMap<String, String>> = HashMap::new();
        for (namespace, ids) in ids_by_namespace {
            let values = found.entry(namespace).or_default();
            for chunk in ids.chunks(BATCH_SIZE) {
                let mut query = QueryBuilder::<Sqlite>::new(
                    "SELECT key, value FROM universal_cache WHERE namespace = ",
                );
                query
                    .push_bind(namespace)
                    .push(" AND expires_at > ")
                    .push_bind(now)
                    .push(" AND key IN (");
                let mut list = query.separated(", ");
                for id in chunk {
                    list.push_bind(*id);
                }
                list.push_unseparated(")");

                let Ok(rows) = query.build().fetch_all(&self.pool).await else {
                    continue;
                };
                values.extend(rows.into_iter().map(|row| (row.get(0), row.get(1))));
            }
        }

        keys.iter()
            .map(|key| {
                let value_json = found.get(key.namespace())?.get(key.id().as_ref())?;
                serde_json::from_str(value_json).ok()
            })
            .collect()
    }

    /// All in one transaction.
    async fn set_many<T: Serialize + Send + Sync>(
        &self,
        entries: &[(CacheKey<T>, T)],
        ttl: Duration,
    ) -> bool {
        if entries.is_empty() {
            return true;
        }
        let expires_at = expires_at(ttl);

        let Ok(mut tx) = self.pool.begin().await else {
            return false;
        };
        let mut written = true;
        for (key, value) in entries {
            written &= upsert(&mut *tx, key, value, expires_at).await;
        }
        tx.commit().await.is_ok() && written
    }

    async fn exists<T: Send + Sync>(&self, key: &CacheKey<T>) -> bool {
        let now = Utc::now();

        sqlx::query(
            "SELECT 1 FROM universal_cache 
             WHERE namespace = ? AND key = ? AND expires_at > ? LIMIT 1",
        )
        .bind(key.namespace())
        .bind(key.id().as_ref())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .is_ok_and(|res| res.is_some())
    }

    async fn invalidate<T: Send + Sync>(&self, key: &CacheKey<T>) {
        let namespace = key.namespace();
        let id = key.id().as_ref();

        let _ = sqlx::query!(
            "DELETE FROM universal_cache WHERE namespace = ? AND key = ?",
            namespace,
            id,
        )
        .execute(&self.pool)
        .await;
    }

    async fn purge_expired(&self) -> u64 {
        let now = Utc::now();

        sqlx::query("DELETE FROM universal_cache WHERE expires_at <= ?")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_or(0, |result| result.rows_affected())
    }
}

/// The rows nobody refreshed for their whole life.
#[async_trait]
impl Housekeeping for SqliteCache {
    fn what(&self) -> &'static str {
        "expired cache rows"
    }

    async fn run(&self) -> u64 {
        self.purge_expired().await
    }
}

/// Every row, whatever it holds: all of `universal_cache` is fetched again when needed.
#[async_trait]
impl CacheCleaner for SqliteCache {
    fn what(&self) -> &'static str {
        "cache rows"
    }

    fn kind(&self) -> CacheKind {
        CacheKind::Metadata
    }

    async fn size(&self) -> u64 {
        sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(SUM(LENGTH(namespace) + LENGTH(key) + LENGTH(value)), 0)
             FROM universal_cache",
        )
        .fetch_one(&self.pool)
        .await
        .map_or(0, |bytes| u64::try_from(bytes).unwrap_or_default())
    }

    async fn clear(&self) -> u64 {
        sqlx::query("DELETE FROM universal_cache")
            .execute(&self.pool)
            .await
            .map_or(0, |result| result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{pool::PoolOptions, sqlite::SqliteConnectOptions};

    use crate::{core::migrations::run_migrations, shared::cache::domain::CacheId};

    use super::*;

    const LONG: Duration = Duration::from_hours(1);

    async fn create_cache() -> SqliteCache {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);
        let pool: SqlitePool = PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to create in-memory SQLite pool");
        run_migrations(&pool).await;

        SqliteCache::new(pool)
    }

    fn key(namespace: &'static str, id: &str) -> CacheKey<String> {
        CacheKey::new(namespace, CacheId::Named(id.to_owned()))
    }

    fn entry(namespace: &'static str, id: &str, value: &str) -> (CacheKey<String>, String) {
        (key(namespace, id), value.to_owned())
    }

    #[tokio::test]
    async fn should_return_the_values_in_the_keys_order_with_none_for_misses_when_getting_many() {
        let cache = create_cache().await;
        cache
            .set_many(&[entry("a", "1", "one"), entry("a", "2", "two")], LONG)
            .await;
        cache.set(&key("b", "1"), &"b-one".to_owned(), LONG).await;

        let found = cache
            .get_many(&[
                key("a", "2"),
                key("a", "missing"),
                key("b", "1"),
                key("a", "1"),
                key("b", "2"),
            ])
            .await;

        assert_eq!(
            found,
            [
                Some("two".to_owned()),
                None,
                Some("b-one".to_owned()),
                Some("one".to_owned()),
                None,
            ]
        );
        assert_eq!(cache.get(&key("a", "1")).await, Some("one".to_owned()));
    }

    #[tokio::test]
    async fn should_hide_an_expired_value_when_getting_many() {
        let cache = create_cache().await;
        cache
            .set_many(&[entry("a", "old", "old")], Duration::ZERO)
            .await;
        cache.set_many(&[entry("a", "new", "new")], LONG).await;

        let found = cache.get_many(&[key("a", "old"), key("a", "new")]).await;

        assert_eq!(found, [None, Some("new".to_owned())]);
    }

    #[tokio::test]
    async fn should_replace_a_value_and_its_expiry_when_set_many_again() {
        let cache = create_cache().await;
        cache.set(&key("a", "1"), &"first".to_owned(), LONG).await;
        cache.set(&key("a", "2"), &"live".to_owned(), LONG).await;

        cache.set_many(&[entry("a", "1", "second")], LONG).await;
        cache
            .set_many(&[entry("a", "2", "expired")], Duration::ZERO)
            .await;
        let found = cache.get_many(&[key("a", "1"), key("a", "2")]).await;

        assert_eq!(found, [Some("second".to_owned()), None]);
    }

    #[tokio::test]
    async fn should_read_and_write_all_keys_when_they_span_several_batches() {
        let cache = create_cache().await;
        let count = BATCH_SIZE * 2 + 1;
        let entries: Vec<_> = (0..count)
            .map(|i| entry("a", &i.to_string(), &format!("value {i}")))
            .collect();

        cache.set_many(&entries, LONG).await;
        let mut keys: Vec<_> = (0..count).map(|i| key("a", &i.to_string())).collect();
        keys.push(key("a", "missing"));
        let found = cache.get_many(&keys).await;

        assert_eq!(found.len(), count + 1);
        for (i, value) in found[..count].iter().enumerate() {
            assert_eq!(value.as_deref(), Some(format!("value {i}").as_str()));
        }
        assert_eq!(found[count], None);
    }

    #[tokio::test]
    async fn should_delete_only_expired_values_when_purging() {
        let cache = create_cache().await;
        cache
            .set(&key("a", "old"), &"old".to_owned(), Duration::ZERO)
            .await;
        cache
            .set(&key("a", "old2"), &"old2".to_owned(), Duration::ZERO)
            .await;
        cache.set(&key("a", "new"), &"new".to_owned(), LONG).await;

        let purged = cache.purge_expired().await;

        assert_eq!(purged, 2);
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM universal_cache")
            .fetch_one(&cache.pool)
            .await
            .unwrap();
        assert_eq!(rows, 1);
        assert_eq!(cache.get(&key("a", "new")).await, Some("new".to_owned()));
    }

    #[tokio::test]
    async fn should_remove_every_row_when_cleared() {
        let cache = create_cache().await;
        cache
            .set_many(&[entry("a", "1", "one"), entry("b", "2", "two")], LONG)
            .await;

        assert_eq!(CacheCleaner::clear(&cache).await, 2);

        assert_eq!(cache.get(&key("a", "1")).await, None);
        assert_eq!(cache.get(&key("b", "2")).await, None);
    }

    #[tokio::test]
    async fn should_do_nothing_when_given_no_keys() {
        let cache = create_cache().await;

        cache.set_many::<String>(&[], LONG).await;
        let found = cache.get_many::<String>(&[]).await;

        assert!(found.is_empty());
    }
}
