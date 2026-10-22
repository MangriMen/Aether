use async_trait::async_trait;
use sqlx::{Sqlite, SqliteConnection, SqlitePool, Transaction};

use crate::features::settings::{
    DefaultInstanceSettings, DefaultInstanceSettingsStorage, Hooks, MemorySettings, SettingsError,
    WindowSettings, WindowSize,
};

pub struct SqliteDefaultInstanceSettingsStorage {
    pool: SqlitePool,
}

impl SqliteDefaultInstanceSettingsStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// A read-modify-write has to hold the write lock from the read on, or a concurrent write
    /// commits in between and is overwritten. `BEGIN IMMEDIATE` takes that lock up front.
    async fn begin_write(&self) -> Result<Transaction<'static, Sqlite>, SettingsError> {
        Ok(self.pool.begin_with("BEGIN IMMEDIATE").await?)
    }

    pub async fn exists(&self) -> bool {
        sqlx::query!("SELECT 1 as exists_flag FROM default_instance_settings WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .is_ok_and(|row| row.is_some())
    }
}

#[async_trait]
impl DefaultInstanceSettingsStorage for SqliteDefaultInstanceSettingsStorage {
    async fn get(&self) -> Result<DefaultInstanceSettings, SettingsError> {
        let mut conn = self.pool.acquire().await?;
        read_settings(&mut conn).await
    }

    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        let mut tx = self.begin_write().await?;
        let settings = write_settings(&mut tx, settings).await?;
        tx.commit().await?;
        Ok(settings)
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(DefaultInstanceSettings) -> (DefaultInstanceSettings, bool) + Send>,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        let mut tx = self.begin_write().await?;
        // A failed read must not fall back to the defaults: the edit would overwrite the
        // stored settings with them.
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

async fn read_settings(
    conn: &mut SqliteConnection,
) -> Result<DefaultInstanceSettings, SettingsError> {
    let row = sqlx::query!(
        r#"SELECT 
                launch_args_json, env_vars_json, max_memory, force_fullscreen, 
                window_width, window_height, hook_pre_launch, hook_wrapper, hook_post_exit
               FROM default_instance_settings WHERE id = 1"#
    )
    .fetch_optional(&mut *conn)
    .await?;

    let Some(row) = row else {
        return Ok(DefaultInstanceSettings::default());
    };

    let launch_args = serde_json::from_str(&row.launch_args_json).unwrap_or_default();
    let env_vars = serde_json::from_str(&row.env_vars_json).unwrap_or_default();

    let max_memory = u32::try_from(row.max_memory);
    let memory = if let Ok(max_memory) = max_memory {
        MemorySettings::new(max_memory)
    } else {
        MemorySettings::default()
    };

    let width = u16::try_from(row.window_width);
    let height = u16::try_from(row.window_height);

    let window_size = if let Ok(width) = width
        && let Ok(height) = height
    {
        WindowSize::new(width, height)
    } else {
        WindowSize::default()
    };

    Ok(DefaultInstanceSettings::new(
        launch_args,
        env_vars,
        memory,
        WindowSettings::new(row.force_fullscreen, window_size),
        Hooks::new(row.hook_pre_launch, row.hook_wrapper, row.hook_post_exit),
    ))
}

async fn write_settings(
    conn: &mut SqliteConnection,
    settings: DefaultInstanceSettings,
) -> Result<DefaultInstanceSettings, SettingsError> {
    let launch_args_json =
        serde_json::to_string(settings.launch_args()).unwrap_or_else(|_| "[]".into());
    let env_vars_json = serde_json::to_string(settings.env_vars()).unwrap_or_else(|_| "[]".into());

    let memory_max = i64::from(settings.memory().maximum);
    let fullscreen = settings.window().force_fullscreen();
    let width = i64::from(settings.window().game_resolution().width());
    let height = i64::from(settings.window().game_resolution().height());

    let pre_launch = settings.hooks().pre_launch().to_string();
    let wrapper = settings.hooks().wrapper().to_string();
    let post_exit = settings.hooks().post_exit().to_string();

    sqlx::query!(
        r#"INSERT INTO default_instance_settings (
                id, launch_args_json, env_vars_json, max_memory, force_fullscreen, 
                window_width, window_height, hook_pre_launch, hook_wrapper, hook_post_exit
            ) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                launch_args_json = excluded.launch_args_json,
                env_vars_json = excluded.env_vars_json,
                max_memory = excluded.max_memory,
                force_fullscreen = excluded.force_fullscreen,
                window_width = excluded.window_width,
                window_height = excluded.window_height,
                hook_pre_launch = excluded.hook_pre_launch,
                hook_wrapper = excluded.hook_wrapper,
                hook_post_exit = excluded.hook_post_exit"#,
        launch_args_json,
        env_vars_json,
        memory_max,
        fullscreen,
        width,
        height,
        pre_launch,
        wrapper,
        post_exit
    )
    .execute(&mut *conn)
    .await?;

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::*;

    /// A file database, not `:memory:`: every connection to an in-memory database gets its own
    /// copy, so the pool would serialize the updates instead of the database.
    async fn storage(dir: &tempfile::TempDir) -> SqliteDefaultInstanceSettingsStorage {
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
        SqliteDefaultInstanceSettingsStorage::new(pool)
    }

    #[tokio::test]
    async fn should_keep_every_launch_arg_added_by_concurrent_updates() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Arc::new(storage(&dir).await);

        let tasks: Vec<_> = (0..20)
            .map(|i| {
                let storage = storage.clone();
                tokio::spawn(async move {
                    storage
                        .update_mut(Box::new(move |mut settings| {
                            let mut args = settings.launch_args().to_vec();
                            args.push(format!("-Darg={i}"));
                            let changed = settings.update_launch_args(args);
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

        assert_eq!(storage.get().await.unwrap().launch_args().len(), 20);
    }

    #[tokio::test]
    async fn should_not_apply_the_edit_when_the_read_fails() {
        let dir = tempfile::tempdir().unwrap();
        let storage = storage(&dir).await;
        sqlx::query("DROP TABLE default_instance_settings")
            .execute(&storage.pool)
            .await
            .unwrap();

        let edited = Arc::new(AtomicBool::new(false));
        let flag = edited.clone();
        let result = storage
            .update_mut(Box::new(move |settings| {
                flag.store(true, Ordering::SeqCst);
                (settings, true)
            }))
            .await;

        assert!(result.is_err());
        assert!(!edited.load(Ordering::SeqCst));
    }
}
