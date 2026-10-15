use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::features::settings::Settings;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingsV1 {
    launcher_dir: PathBuf,
    metadata_dir: PathBuf,

    max_concurrent_downloads: usize,

    enabled_plugins: HashSet<String>,
}

impl From<SettingsV1> for Settings {
    fn from(value: SettingsV1) -> Self {
        Self::new(value.max_concurrent_downloads, value.enabled_plugins)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsV2 {
    pub max_concurrent_downloads: usize,

    pub enabled_plugins: HashSet<String>,

    #[serde(default)]
    pub platform_providers: HashMap<String, String>,
}

impl From<SettingsV2> for Settings {
    fn from(value: SettingsV2) -> Self {
        Self::new(value.max_concurrent_downloads, value.enabled_plugins)
            .with_platform_providers(value.platform_providers)
    }
}

impl From<&Settings> for SettingsV2 {
    fn from(value: &Settings) -> Self {
        Self {
            max_concurrent_downloads: value.max_concurrent_downloads(),
            enabled_plugins: value.enabled_plugins().clone(),
            platform_providers: value.platform_providers().clone(),
        }
    }
}
