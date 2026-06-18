use aether_core::features::settings::{EditSettings, Settings};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use specta::Type;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    max_concurrent_downloads: usize,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditSettingsDto {
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
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
