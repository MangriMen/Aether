use aether_core::features::plugins::{PluginSettings, app::EditPluginSettings};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::plugins::PathMappingDto;

#[derive(Serialize, Deserialize, Debug, Default, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettingsDto {
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    #[serde(default)]
    pub allowed_paths: Vec<PathMappingDto>,
}

#[derive(Serialize, Deserialize, Debug, Default, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditPluginSettingsDto {
    #[specta(optional)]
    pub allowed_hosts: Option<Vec<String>>,
    #[specta(optional)]
    pub allowed_paths: Option<Vec<PathMappingDto>>,
}

impl From<PluginSettings> for PluginSettingsDto {
    fn from(value: PluginSettings) -> Self {
        Self {
            allowed_hosts: value.allowed_hosts,
            allowed_paths: value.allowed_paths.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<EditPluginSettingsDto> for EditPluginSettings {
    fn from(value: EditPluginSettingsDto) -> Self {
        Self {
            allowed_hosts: value.allowed_hosts,
            allowed_paths: value
                .allowed_paths
                .map(|paths| paths.into_iter().map(Into::into).collect()),
        }
    }
}
