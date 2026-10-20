use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::features::settings::{
    DEFAULT_MAX_CONCURRENT_DOWNLOADS, DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64, Settings,
    SettingsError, SettingsStorage,
};

pub struct SqliteSettingsStorage {
    pool: SqlitePool,
    /// `upsert` rewrites every table from a snapshot, so a write that started from a stale
    /// snapshot would drop a concurrent change. Writes go one at a time.
    write_lock: tokio::sync::Mutex<()>,
}

impl SqliteSettingsStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            write_lock: tokio::sync::Mutex::new(()),
        }
    }
}

#[async_trait]
impl SettingsStorage for SqliteSettingsStorage {
    async fn get(&self) -> Result<Settings, SettingsError> {
        let base =
            sqlx::query!("SELECT max_concurrent_downloads FROM launcher_settings WHERE id = 1")
                .fetch_optional(&self.pool)
                .await?;

        let Some(base) = base else {
            return Ok(Settings::default());
        };

        let plugins = sqlx::query!("SELECT plugin_id FROM enabled_plugins")
            .fetch_all(&self.pool)
            .await?;

        let enabled_plugins = plugins.into_iter().map(|p| p.plugin_id).collect();

        let platform_providers =
            sqlx::query!("SELECT platform, provider_id FROM platform_providers")
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| (row.platform, row.provider_id))
                .collect();

        Ok(Settings::new(
            base.max_concurrent_downloads
                .try_into()
                .unwrap_or(DEFAULT_MAX_CONCURRENT_DOWNLOADS),
            enabled_plugins,
        )
        .with_platform_providers(platform_providers))
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
        let _guard = self.write_lock.lock().await;
        self.write(settings).await
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
    ) -> Result<Settings, SettingsError> {
        let _guard = self.write_lock.lock().await;
        let settings = self.get().await?;
        let (settings, changed) = f(settings);
        if changed {
            self.write(settings).await
        } else {
            Ok(settings)
        }
    }
}

impl SqliteSettingsStorage {
    async fn write(&self, settings: Settings) -> Result<Settings, SettingsError> {
        let mut tx = self.pool.begin().await?;

        let max_downloads = i64::try_from(settings.max_concurrent_downloads())
            .unwrap_or(DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64);

        // Update main table
        sqlx::query!(
            "INSERT INTO launcher_settings (id, max_concurrent_downloads)
             VALUES (1, ?)
             ON CONFLICT(id) DO UPDATE SET
            max_concurrent_downloads = excluded.max_concurrent_downloads",
            max_downloads
        )
        .execute(&mut *tx)
        .await?;

        // Sync enabled_plugins: Delete removed and insert new
        sqlx::query!("DELETE FROM enabled_plugins")
            .execute(&mut *tx)
            .await?;

        for plugin_id in settings.enabled_plugins() {
            sqlx::query!(
                "INSERT INTO enabled_plugins (plugin_id) VALUES (?)",
                plugin_id
            )
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query!("DELETE FROM platform_providers")
            .execute(&mut *tx)
            .await?;

        for (platform, provider_id) in settings.platform_providers() {
            sqlx::query!(
                "INSERT INTO platform_providers (platform, provider_id) VALUES (?, ?)",
                platform,
                provider_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn storage() -> SqliteSettingsStorage {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(false);
        let pool = sqlx::pool::PoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        crate::core::migrations::run_migrations(&pool)
            .await
            .unwrap();
        SqliteSettingsStorage::new(pool)
    }

    #[tokio::test]
    async fn should_keep_every_plugin_enabled_by_concurrent_updates() {
        let storage = std::sync::Arc::new(storage().await);

        let tasks: Vec<_> = (0..20)
            .map(|i| {
                let storage = storage.clone();
                tokio::spawn(async move {
                    storage
                        .update_mut(Box::new(move |mut settings| {
                            let changed = settings.enable_plugin(&format!("plugin-{i}"));
                            (settings, changed)
                        }))
                        .await
                        .unwrap();
                })
            })
            .collect();
        for task in tasks {
            task.await.unwrap();
        }

        assert_eq!(storage.get().await.unwrap().enabled_plugins().len(), 20);
    }
}
