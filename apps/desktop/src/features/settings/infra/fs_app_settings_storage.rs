use std::path::PathBuf;

use aether_core::shared::json_store::infra::JsonValueStore;
use async_trait::async_trait;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::features::settings::{
    ActionOnInstanceLaunch, AppSettings, AppSettingsError, AppSettingsStorage, WindowEffect,
};

/// DTO for `AppSettings` serialization in FS storage.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsV1 {
    pub action_on_instance_launch: ActionOnInstanceLaunchV1,
    pub is_actual_transparent: bool,
    pub transparent: bool,
    pub window_effect: WindowEffectV1,
}

impl Default for AppSettingsV1 {
    fn default() -> Self {
        Self {
            action_on_instance_launch: ActionOnInstanceLaunchV1::Nothing,
            is_actual_transparent: false,
            transparent: false,
            window_effect: WindowEffectV1::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOnInstanceLaunchV1 {
    Nothing,
    Hide,
    Close,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowEffectV1 {
    Off,
    MicaLight,
    MicaDark,
    Mica,
    Acrylic,
}

impl From<AppSettings> for AppSettingsV1 {
    fn from(v: AppSettings) -> Self {
        Self {
            action_on_instance_launch: match v.action_on_instance_launch {
                ActionOnInstanceLaunch::Nothing => ActionOnInstanceLaunchV1::Nothing,
                ActionOnInstanceLaunch::Hide => ActionOnInstanceLaunchV1::Hide,
                ActionOnInstanceLaunch::Close => ActionOnInstanceLaunchV1::Close,
            },
            is_actual_transparent: v.is_actual_transparent,
            transparent: v.transparent,
            window_effect: match v.window_effect {
                WindowEffect::Off => WindowEffectV1::Off,
                WindowEffect::MicaLight => WindowEffectV1::MicaLight,
                WindowEffect::MicaDark => WindowEffectV1::MicaDark,
                WindowEffect::Mica => WindowEffectV1::Mica,
                WindowEffect::Acrylic => WindowEffectV1::Acrylic,
            },
        }
    }
}

impl From<AppSettingsV1> for AppSettings {
    fn from(v: AppSettingsV1) -> Self {
        Self {
            action_on_instance_launch: match v.action_on_instance_launch {
                ActionOnInstanceLaunchV1::Nothing => ActionOnInstanceLaunch::Nothing,
                ActionOnInstanceLaunchV1::Hide => ActionOnInstanceLaunch::Hide,
                ActionOnInstanceLaunchV1::Close => ActionOnInstanceLaunch::Close,
            },
            is_actual_transparent: v.is_actual_transparent,
            transparent: v.transparent,
            window_effect: match v.window_effect {
                WindowEffectV1::Off => WindowEffect::Off,
                WindowEffectV1::MicaLight => WindowEffect::MicaLight,
                WindowEffectV1::MicaDark => WindowEffect::MicaDark,
                WindowEffectV1::Mica => WindowEffect::Mica,
                WindowEffectV1::Acrylic => WindowEffect::Acrylic,
            },
        }
    }
}

pub struct FsAppSettingsStorage {
    store: JsonValueStore<AppSettingsV1>,
    path: PathBuf,
}

impl FsAppSettingsStorage {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            store: JsonValueStore::new(path.clone()),
            path,
        }
    }

    pub fn get_file_path(&self) -> PathBuf {
        self.path.clone()
    }
}

#[async_trait]
impl AppSettingsStorage for FsAppSettingsStorage {
    async fn get(&self) -> Result<AppSettings, AppSettingsError> {
        let dto: AppSettingsV1 = self.store.read_or_default().await.unwrap_or_else(|err| {
            warn!("Failed to read settings: {err}");
            AppSettingsV1::default()
        });
        Ok(dto.into())
    }

    async fn upsert(&self, settings: AppSettings) -> Result<(), AppSettingsError> {
        let dto: AppSettingsV1 = settings.into();
        Ok(self
            .store
            .write(&dto)
            .await
            .map_err(|err| AppSettingsError::Storage(err.to_string()))?)
    }
}
