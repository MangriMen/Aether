use async_trait::async_trait;
use sqlx::SqlitePool;
use tracing::warn;

use crate::{
    features::settings::{
        DefaultInstanceSettings, DefaultInstanceSettingsStorage, Hooks, MemorySettings,
        SettingsError, WindowSize,
    },
    shared::UpdateAction,
};

pub struct SqliteDefaultInstanceSettingsStorage {
    pool: SqlitePool,
}

impl SqliteDefaultInstanceSettingsStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
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
        // Fetch main settings
        let row = sqlx::query!(
            "SELECT max_memory, force_fullscreen, window_width, window_height, 
                    pre_launch_hook, wrapper_hook, post_exit_hook 
             FROM default_instance_settings WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(DefaultInstanceSettings::default());
        };

        // Fetch collections
        let launch_args = sqlx::query!("SELECT arg FROM launch_args")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| r.arg)
            .collect();

        let env_vars = sqlx::query!("SELECT key, value FROM env_vars")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| (r.key, r.value))
            .collect();

        // Mapping to struct
        Ok(DefaultInstanceSettings::new(
            launch_args,
            env_vars,
            map_memory(row.max_memory),
            map_window_size(row.window_width, row.window_height),
            row.force_fullscreen,
            Hooks::new(row.pre_launch_hook, row.wrapper_hook, row.post_exit_hook),
        ))
    }

    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        let mut tx = self.pool.begin().await?;

        let max_memory = settings.memory().maximum;
        let force_fullscreen = settings.force_fullscreen();
        let width = settings.game_resolution().width();
        let height = settings.game_resolution().height();
        let pre_launch = settings.hooks().pre_launch();
        let wrapper = settings.hooks().wrapper();
        let post_exit = settings.hooks().post_exit();

        // Update main table
        sqlx::query!(
            "INSERT INTO default_instance_settings (
                id, max_memory, force_fullscreen, window_width, window_height, 
                pre_launch_hook, wrapper_hook, post_exit_hook
             ) VALUES (1, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                max_memory = excluded.max_memory,
                force_fullscreen = excluded.force_fullscreen,
                window_width = excluded.window_width,
                window_height = excluded.window_height,
                pre_launch_hook = excluded.pre_launch_hook,
                wrapper_hook = excluded.wrapper_hook,
                post_exit_hook = excluded.post_exit_hook",
            max_memory,
            force_fullscreen,
            width,
            height,
            pre_launch,
            wrapper,
            post_exit,
        )
        .execute(&mut *tx)
        .await?;

        // Sync launch args
        sqlx::query!("DELETE FROM launch_args")
            .execute(&mut *tx)
            .await?;
        for arg in settings.launch_args() {
            sqlx::query!("INSERT INTO launch_args (arg) VALUES (?)", arg)
                .execute(&mut *tx)
                .await?;
        }

        // Sync env vars
        sqlx::query!("DELETE FROM env_vars")
            .execute(&mut *tx)
            .await?;
        for (key, value) in settings.env_vars() {
            sqlx::query!(
                "INSERT INTO env_vars (key, value) VALUES (?, ?)",
                key,
                value
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(settings)
    }

    async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut DefaultInstanceSettings) -> UpdateAction<R> + Send,
    {
        let mut settings = self.get().await.unwrap_or_default();

        match f(&mut settings) {
            UpdateAction::Save(result) => {
                self.upsert(settings).await?;
                Ok(result)
            }
            UpdateAction::NoChanges(result) => Ok(result),
        }
    }
}

fn map_window_size(width: i64, height: i64) -> WindowSize {
    let Ok(width) = u16::try_from(width) else {
        warn!("Invalid window width: {}. Using default.", width);
        return WindowSize::default();
    };

    let Ok(height) = u16::try_from(height) else {
        warn!("Invalid window height: {}. Using default.", height);
        return WindowSize::default();
    };

    WindowSize::new(width, height)
}

fn map_memory(val: i64) -> MemorySettings {
    u32::try_from(val).map_or_else(
        |err| {
            warn!("Failed to map memory settings: {}. Using default", err);
            MemorySettings::default()
        },
        MemorySettings::new,
    )
}
