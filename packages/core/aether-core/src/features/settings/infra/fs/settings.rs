use std::{collections::HashSet, path::PathBuf};

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
