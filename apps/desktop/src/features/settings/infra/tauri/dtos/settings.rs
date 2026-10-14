use std::collections::HashMap;

use aether_core::features::settings::{EditPlatformProvider, EditSettings, Settings};
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

    /// Platform → id of the provider chosen to serve it.
    platform_providers: HashMap<String, String>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditSettingsDto {
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub max_concurrent_downloads: usize,

    /// Choose the provider serving a platform; absent leaves the choices as they are.
    #[serde(default)]
    pub platform_provider: Option<EditPlatformProviderDto>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditPlatformProviderDto {
    pub platform: String,

    /// `None` drops the choice, so the platform falls back to the default rule.
    pub provider_id: Option<String>,
}

impl From<Settings> for SettingsDto {
    fn from(value: Settings) -> Self {
        Self {
            max_concurrent_downloads: value.max_concurrent_downloads(),
            platform_providers: value.platform_providers().clone(),
        }
    }
}

impl From<EditSettingsDto> for EditSettings {
    fn from(value: EditSettingsDto) -> Self {
        Self {
            max_concurrent_downloads: value.max_concurrent_downloads,
            platform_provider: value.platform_provider.map(|choice| EditPlatformProvider {
                platform: choice.platform,
                provider_id: choice.provider_id,
            }),
        }
    }
}
