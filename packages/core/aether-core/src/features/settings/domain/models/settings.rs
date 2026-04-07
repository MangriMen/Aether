use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    launcher_dir: PathBuf,
    metadata_dir: PathBuf,

    max_concurrent_downloads: usize,

    enabled_plugins: HashSet<String>,
}

impl Settings {
    pub fn new(
        launcher_dir: PathBuf,
        metadata_dir: PathBuf,
        max_concurrent_downloads: usize,
        enabled_plugins: HashSet<String>,
    ) -> Self {
        Self {
            launcher_dir,
            metadata_dir,
            max_concurrent_downloads,
            enabled_plugins,
        }
    }

    pub fn from_dirs(launcher_dir: PathBuf, metadata_dir: PathBuf) -> Self {
        Self {
            launcher_dir,
            metadata_dir,
            max_concurrent_downloads: 10,
            enabled_plugins: HashSet::default(),
        }
    }

    pub fn launcher_dir(&self) -> &Path {
        &self.launcher_dir
    }

    pub fn metadata_dir(&self) -> &Path {
        &self.metadata_dir
    }

    pub fn max_concurrent_downloads(&self) -> usize {
        self.max_concurrent_downloads
    }

    pub fn enabled_plugins(&self) -> &HashSet<String> {
        &self.enabled_plugins
    }

    pub fn set_max_concurrent_downloads(&mut self, max_concurrent_downloads: usize) {
        self.max_concurrent_downloads = max_concurrent_downloads
    }

    pub fn is_plugin_enabled(&self, plugin_id: &str) -> bool {
        self.enabled_plugins.contains(plugin_id)
    }

    pub fn enable_plugin(&mut self, plugin_id: &str) -> bool {
        if !self.enabled_plugins.contains(plugin_id) {
            return self.enabled_plugins.insert(plugin_id.to_owned());
        }

        false
    }

    pub fn disable_plugin(&mut self, plugin_id: &str) -> bool {
        self.enabled_plugins.remove(plugin_id)
    }
}
