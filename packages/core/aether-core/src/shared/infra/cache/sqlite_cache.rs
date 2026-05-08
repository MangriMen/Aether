use crate::shared::{Cache, CacheKey};
use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Row, SqlitePool};
use std::time::Duration;

pub struct SqliteCache {
    pool: SqlitePool,
}

impl SqliteCache {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
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
        let expires_at =
            Utc::now() + ChronoDuration::from_std(ttl).unwrap_or_else(|_| ChronoDuration::zero());

        let Ok(value_json) = serde_json::to_string(value) else {
            return;
        };

        let namespace = key.namespace();
        let id = key.id().as_ref();

        let _ = sqlx::query!(
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
        .execute(&self.pool)
        .await;
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
}
