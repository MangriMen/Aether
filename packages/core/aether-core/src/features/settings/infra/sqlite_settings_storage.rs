use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{
    features::settings::{
        DEFAULT_MAX_CONCURRENT_DOWNLOADS, DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64, Settings,
        SettingsError, SettingsStorage,
    },
    shared::UpdateAction,
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
        let base = sqlx::query!(
            "SELECT launcher_dir, metadata_dir, max_concurrent_downloads FROM launcher_settings WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        let base = base.ok_or(SettingsError::NotFound)?;

        let plugins = sqlx::query!("SELECT plugin_id FROM enabled_plugins")
            .fetch_all(&self.pool)
            .await?;

        let enabled_plugins = plugins.into_iter().map(|p| p.plugin_id).collect();

        Ok(Settings::new(
            base.launcher_dir.into(),
            base.metadata_dir.into(),
            base.max_concurrent_downloads
                .try_into()
                .unwrap_or(DEFAULT_MAX_CONCURRENT_DOWNLOADS),
            enabled_plugins,
        ))
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
        let mut tx = self.pool.begin().await?;

        let launcher_dir = settings.launcher_dir().to_string_lossy().to_string();
        let metadata_dir = settings.metadata_dir().to_string_lossy().to_string();
        let max_downloads = i64::try_from(settings.max_concurrent_downloads())
            .unwrap_or(DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64);

        // Update main table
        sqlx::query!(
            "INSERT INTO launcher_settings (id, launcher_dir, metadata_dir, max_concurrent_downloads)
             VALUES (1, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                launcher_dir = excluded.launcher_dir,
                metadata_dir = excluded.metadata_dir,
                max_concurrent_downloads = excluded.max_concurrent_downloads",
            launcher_dir,
            metadata_dir,
            max_downloads
        )
        .execute(&mut *tx)
        .await?;

        // Sync plugins: Delete removed and insert new
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

    async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut Settings) -> UpdateAction<R> + Send,
    {
        let mut settings = self.get().await?;

        match f(&mut settings) {
            UpdateAction::Save(result) => {
                self.upsert(settings).await?;
                Ok(result)
            }
            UpdateAction::NoChanges(result) => Ok(result),
        }
    }
}

impl From<sqlx::Error> for SettingsError {
    fn from(value: sqlx::Error) -> Self {
        SettingsError::Storage(value.to_string())
    }
}
