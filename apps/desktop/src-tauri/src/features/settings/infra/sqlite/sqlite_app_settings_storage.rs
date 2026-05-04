use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::features::settings::{
    ActionOnInstanceLaunch, AppSettings, AppSettingsError, AppSettingsStorage, WindowEffect,
};

pub struct SqliteAppSettingsStorage {
    pool: SqlitePool,
}

impl SqliteAppSettingsStorage {
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn exists(&self) -> bool {
        sqlx::query!("SELECT 1 as exists_flag FROM app_settings WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .is_ok_and(|row| row.is_some())
    }
}

#[async_trait]
impl AppSettingsStorage for SqliteAppSettingsStorage {
    async fn get(&self) -> Result<AppSettings, AppSettingsError> {
        let row = sqlx::query_as!(SqlAppSettings, "SELECT * FROM app_settings WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => AppSettings::try_from(row),
            None => Ok(AppSettings::default()),
        }
    }

    async fn upsert(&self, settings: AppSettings) -> Result<(), AppSettingsError> {
        let sql = SqlAppSettings::from(settings);

        sqlx::query!(
            "INSERT INTO app_settings (id, action_on_instance_launch, is_actual_transparent, transparent, window_effect)
             VALUES (1, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                action_on_instance_launch = excluded.action_on_instance_launch,
                is_actual_transparent = excluded.is_actual_transparent,
                transparent = excluded.transparent,
                window_effect = excluded.window_effect",
            sql.action_on_instance_launch,
            sql.is_actual_transparent,
            sql.transparent,
            sql.window_effect
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SqlAppSettings {
    #[allow(unused)]
    id: i64,
    action_on_instance_launch: String,
    is_actual_transparent: bool,
    transparent: bool,
    window_effect: String,
}

impl From<AppSettings> for SqlAppSettings {
    fn from(s: AppSettings) -> Self {
        Self {
            id: 1,
            action_on_instance_launch: match s.action_on_instance_launch {
                ActionOnInstanceLaunch::Nothing => "nothing".to_string(),
                ActionOnInstanceLaunch::Hide => "hide".to_string(),
                ActionOnInstanceLaunch::Close => "close".to_string(),
            },
            is_actual_transparent: s.is_actual_transparent,
            transparent: s.transparent,
            window_effect: match s.window_effect {
                WindowEffect::Off => "off".to_string(),
                WindowEffect::MicaLight => "mica_light".to_string(),
                WindowEffect::MicaDark => "mica_dark".to_string(),
                WindowEffect::Mica => "mica".to_string(),
                WindowEffect::Acrylic => "acrylic".to_string(),
            },
        }
    }
}

impl TryFrom<SqlAppSettings> for AppSettings {
    type Error = AppSettingsError;

    fn try_from(value: SqlAppSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            action_on_instance_launch: match value.action_on_instance_launch.as_str() {
                "nothing" => ActionOnInstanceLaunch::Nothing,
                "hide" => ActionOnInstanceLaunch::Hide,
                "close" => ActionOnInstanceLaunch::Close,
                _ => {
                    log::error!(
                        "Unknown action on instance launch in DB: {}, falling back to default",
                        value.window_effect
                    );
                    ActionOnInstanceLaunch::default()
                }
            },
            is_actual_transparent: value.is_actual_transparent,
            transparent: value.transparent,
            window_effect: match value.window_effect.as_str() {
                "off" => WindowEffect::Off,
                "mica_light" => WindowEffect::MicaLight,
                "mica_dark" => WindowEffect::MicaDark,
                "mica" => WindowEffect::Mica,
                "acrylic" => WindowEffect::Acrylic,
                _ => {
                    log::error!(
                        "Unknown window effect in DB: {}, falling back to default",
                        value.window_effect
                    );
                    WindowEffect::default()
                }
            },
        })
    }
}

impl From<sqlx::Error> for AppSettingsError {
    fn from(err: sqlx::Error) -> Self {
        AppSettingsError::Storage(err.to_string())
    }
}
