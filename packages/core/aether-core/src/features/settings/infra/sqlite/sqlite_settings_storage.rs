use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::features::settings::{
    DEFAULT_MAX_CONCURRENT_DOWNLOADS, DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64, Settings,
    SettingsError, SettingsStorage,
};

pub struct SqliteSettingsStorage {
    pool: SqlitePool,
}

impl SqliteSettingsStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
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

        Ok(Settings::new(
            base.max_concurrent_downloads
                .try_into()
                .unwrap_or(DEFAULT_MAX_CONCURRENT_DOWNLOADS),
            enabled_plugins,
        ))
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
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

        tx.commit().await?;

        Ok(settings)
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
    ) -> Result<Settings, SettingsError> {
        let settings = self.get().await?;
        let (settings, changed) = f(settings);
        if changed {
            self.upsert(settings).await
        } else {
            Ok(settings)
        }
    }
}
