use async_trait::async_trait;
use sqlx::SqlitePool;

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
        let row = sqlx::query!(
            r#"SELECT 
                launch_args_json, env_vars_json, max_memory, force_fullscreen, 
                window_width, window_height, hook_pre_launch, hook_wrapper, hook_post_exit
               FROM default_instance_settings WHERE id = 1"#
        )
        .fetch_optional(&self.pool)
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

    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        let launch_args_json =
            serde_json::to_string(settings.launch_args()).unwrap_or_else(|_| "[]".into());
        let env_vars_json =
            serde_json::to_string(settings.env_vars()).unwrap_or_else(|_| "[]".into());

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
        .execute(&self.pool)
        .await?;

        Ok(settings)
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(DefaultInstanceSettings) -> (DefaultInstanceSettings, bool) + Send>,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        let settings = self.get().await.unwrap_or_default();
        let (settings, changed) = f(settings);
        if changed {
            self.upsert(settings).await
        } else {
            Ok(settings)
        }
    }
}
