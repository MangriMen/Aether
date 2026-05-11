use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub const DEFAULT_MAX_CONCURRENT_DOWNLOADS: usize = 10;
#[allow(clippy::cast_possible_wrap)]
pub const DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64: i64 = DEFAULT_MAX_CONCURRENT_DOWNLOADS as i64;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    max_concurrent_downloads: usize,

    enabled_plugins: HashSet<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: DEFAULT_MAX_CONCURRENT_DOWNLOADS,
            enabled_plugins: HashSet::default(),
        }
    }
}

impl Settings {
    pub fn new(max_concurrent_downloads: usize, enabled_plugins: HashSet<String>) -> Self {
        Self {
            max_concurrent_downloads,
            enabled_plugins,
        }
    }

    pub fn max_concurrent_downloads(&self) -> usize {
        self.max_concurrent_downloads
    }

    pub fn enabled_plugins(&self) -> &HashSet<String> {
        &self.enabled_plugins
    }

    pub fn set_max_concurrent_downloads(&mut self, max_concurrent_downloads: usize) {
        self.max_concurrent_downloads = max_concurrent_downloads;
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
