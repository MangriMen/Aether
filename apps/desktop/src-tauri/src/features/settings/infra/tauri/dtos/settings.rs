use aether_core::features::settings::{Settings, app::EditSettings};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    max_concurrent_downloads: usize,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditSettingsDto {
    pub max_concurrent_downloads: usize,
}

impl From<Settings> for SettingsDto {
    fn from(value: Settings) -> Self {
        Self {
            max_concurrent_downloads: value.max_concurrent_downloads(),
        }
    }
}

impl From<EditSettingsDto> for EditSettings {
    fn from(value: EditSettingsDto) -> Self {
        Self {
            max_concurrent_downloads: value.max_concurrent_downloads,
        }
    }
}
