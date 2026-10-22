use async_trait::async_trait;
use sqlx::{Sqlite, SqliteConnection, SqlitePool, Transaction};

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

    /// `upsert` rewrites every table from a snapshot, so a read-modify-write has to hold the
    /// write lock from the read on: a plain `BEGIN` would let another writer commit in between
    /// and be overwritten. `BEGIN IMMEDIATE` takes that lock up front, across connections too.
    async fn begin_write(&self) -> Result<Transaction<'static, Sqlite>, SettingsError> {
        Ok(self.pool.begin_with("BEGIN IMMEDIATE").await?)
    }
}

#[async_trait]
impl SettingsStorage for SqliteSettingsStorage {
    async fn get(&self) -> Result<Settings, SettingsError> {
        let mut conn = self.pool.acquire().await?;
        read_settings(&mut conn).await
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
        let mut tx = self.begin_write().await?;
        let settings = write_settings(&mut tx, settings).await?;
        tx.commit().await?;
        Ok(settings)
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
    ) -> Result<Settings, SettingsError> {
        let mut tx = self.begin_write().await?;
        let settings = read_settings(&mut tx).await?;
        let (settings, changed) = f(settings);
        let settings = if changed {
            write_settings(&mut tx, settings).await?
        } else {
            settings
        };
        tx.commit().await?;
        Ok(settings)
    }
}

async fn read_settings(conn: &mut SqliteConnection) -> Result<Settings, SettingsError> {
    let base = sqlx::query!("SELECT max_concurrent_downloads FROM launcher_settings WHERE id = 1")
        .fetch_optional(&mut *conn)
        .await?;

    let Some(base) = base else {
        return Ok(Settings::default());
    };

    let plugins = sqlx::query!("SELECT plugin_id FROM enabled_plugins")
        .fetch_all(&mut *conn)
        .await?;

    let enabled_plugins = plugins.into_iter().map(|p| p.plugin_id).collect();

    let platform_providers = sqlx::query!("SELECT platform, provider_id FROM platform_providers")
        .fetch_all(&mut *conn)
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

async fn write_settings(
    conn: &mut SqliteConnection,
    settings: Settings,
) -> Result<Settings, SettingsError> {
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
    .execute(&mut *conn)
    .await?;

    // Sync enabled_plugins: Delete removed and insert new
    sqlx::query!("DELETE FROM enabled_plugins")
        .execute(&mut *conn)
        .await?;

    for plugin_id in settings.enabled_plugins() {
        sqlx::query!(
            "INSERT INTO enabled_plugins (plugin_id) VALUES (?)",
            plugin_id
        )
        .execute(&mut *conn)
        .await?;
    }

    sqlx::query!("DELETE FROM platform_providers")
        .execute(&mut *conn)
        .await?;

    for (platform, provider_id) in settings.platform_providers() {
        sqlx::query!(
            "INSERT INTO platform_providers (platform, provider_id) VALUES (?, ?)",
            platform,
            provider_id
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file database, not `:memory:`: every connection to an in-memory database gets its own
    /// copy, so the pool would serialize the updates instead of the database.
    async fn storage(dir: &tempfile::TempDir) -> SqliteSettingsStorage {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(dir.path().join("settings.db"))
            .create_if_missing(true)
            .foreign_keys(false);
        let pool = sqlx::pool::PoolOptions::new()
            .max_connections(5)
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
        let dir = tempfile::tempdir().unwrap();
        let storage = std::sync::Arc::new(storage(&dir).await);

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
