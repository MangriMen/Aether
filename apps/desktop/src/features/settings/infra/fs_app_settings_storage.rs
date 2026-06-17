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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOnInstanceLaunchV1 {
    Nothing,
    Hide,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── AppSettingsV1 conversion tests ──

    #[test]
    fn app_settings_v1_default() {
        let dto = AppSettingsV1::default();
        assert_eq!(
            dto.action_on_instance_launch,
            ActionOnInstanceLaunchV1::Nothing
        );
        assert!(!dto.transparent);
        assert_eq!(dto.window_effect, WindowEffectV1::Off);
    }

    #[test]
    fn app_settings_to_v1_roundtrip_default() {
        let domain = AppSettings::default();
        let dto: AppSettingsV1 = domain.into();
        let back: AppSettings = dto.into();
        assert_eq!(
            back.action_on_instance_launch,
            ActionOnInstanceLaunch::Nothing
        );
        assert!(!back.transparent);
        assert!(!back.is_actual_transparent);
        assert_eq!(back.window_effect, WindowEffect::Off);
    }

    #[test]
    fn app_settings_to_v1_roundtrip_custom() {
        let domain = AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Close,
            is_actual_transparent: true,
            transparent: true,
            window_effect: WindowEffect::Acrylic,
        };
        let dto: AppSettingsV1 = domain.into();
        let back: AppSettings = dto.into();
        assert_eq!(
            back.action_on_instance_launch,
            ActionOnInstanceLaunch::Close
        );
        assert!(back.transparent);
        assert!(back.is_actual_transparent);
        assert_eq!(back.window_effect, WindowEffect::Acrylic);
    }

    // ── ActionOnInstanceLaunchV1 conversion ──

    #[test]
    fn action_v1_domain_roundtrip_all_variants() {
        let domain_variants = [
            ActionOnInstanceLaunch::Nothing,
            ActionOnInstanceLaunch::Hide,
            ActionOnInstanceLaunch::Close,
        ];
        for variant in domain_variants {
            let dto: ActionOnInstanceLaunchV1 = match variant {
                ActionOnInstanceLaunch::Nothing => ActionOnInstanceLaunchV1::Nothing,
                ActionOnInstanceLaunch::Hide => ActionOnInstanceLaunchV1::Hide,
                ActionOnInstanceLaunch::Close => ActionOnInstanceLaunchV1::Close,
            };
            let back = match dto {
                ActionOnInstanceLaunchV1::Nothing => ActionOnInstanceLaunch::Nothing,
                ActionOnInstanceLaunchV1::Hide => ActionOnInstanceLaunch::Hide,
                ActionOnInstanceLaunchV1::Close => ActionOnInstanceLaunch::Close,
            };
            assert_eq!(variant, back);
        }
    }

    // ── WindowEffectV1 conversion ──

    #[test]
    fn window_effect_v1_domain_roundtrip_all_variants() {
        let domain_variants = [
            WindowEffect::Off,
            WindowEffect::MicaLight,
            WindowEffect::MicaDark,
            WindowEffect::Mica,
            WindowEffect::Acrylic,
        ];
        for variant in domain_variants {
            let dto: WindowEffectV1 = match variant {
                WindowEffect::Off => WindowEffectV1::Off,
                WindowEffect::MicaLight => WindowEffectV1::MicaLight,
                WindowEffect::MicaDark => WindowEffectV1::MicaDark,
                WindowEffect::Mica => WindowEffectV1::Mica,
                WindowEffect::Acrylic => WindowEffectV1::Acrylic,
            };
            let back = match dto {
                WindowEffectV1::Off => WindowEffect::Off,
                WindowEffectV1::MicaLight => WindowEffect::MicaLight,
                WindowEffectV1::MicaDark => WindowEffect::MicaDark,
                WindowEffectV1::Mica => WindowEffect::Mica,
                WindowEffectV1::Acrylic => WindowEffect::Acrylic,
            };
            assert_eq!(variant, back);
        }
    }

    // ── JSON serialization ──

    #[test]
    fn app_settings_v1_serialize_camel_case() {
        let dto = AppSettingsV1 {
            action_on_instance_launch: ActionOnInstanceLaunchV1::Hide,
            is_actual_transparent: false,
            transparent: true,
            window_effect: WindowEffectV1::Mica,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("isActualTransparent"), "camelCase: {json}");
        assert!(json.contains("windowEffect"), "camelCase: {json}");
        assert!(json.contains("actionOnInstanceLaunch"), "camelCase: {json}");
    }

    #[test]
    fn app_settings_v1_deserialize_camel_case() {
        let json = r#"{
            "actionOnInstanceLaunch": "hide",
            "isActualTransparent": false,
            "transparent": true,
            "windowEffect": "mica"
        }"#;
        let dto: AppSettingsV1 = serde_json::from_str(json).unwrap();
        assert_eq!(
            dto.action_on_instance_launch,
            ActionOnInstanceLaunchV1::Hide
        );
        assert!(dto.transparent);
        assert_eq!(dto.window_effect, WindowEffectV1::Mica);
    }

    // ── Enum serialization ──

    #[test]
    fn action_v1_serialize_snake_case() {
        assert_eq!(
            serde_json::to_string(&ActionOnInstanceLaunchV1::Nothing).unwrap(),
            "\"nothing\""
        );
        assert_eq!(
            serde_json::to_string(&ActionOnInstanceLaunchV1::Hide).unwrap(),
            "\"hide\""
        );
        assert_eq!(
            serde_json::to_string(&ActionOnInstanceLaunchV1::Close).unwrap(),
            "\"close\""
        );
    }

    #[test]
    fn window_effect_v1_serialize_snake_case() {
        assert_eq!(
            serde_json::to_string(&WindowEffectV1::MicaLight).unwrap(),
            "\"mica_light\""
        );
        assert_eq!(
            serde_json::to_string(&WindowEffectV1::MicaDark).unwrap(),
            "\"mica_dark\""
        );
        assert_eq!(
            serde_json::to_string(&WindowEffectV1::Acrylic).unwrap(),
            "\"acrylic\""
        );
    }
}
