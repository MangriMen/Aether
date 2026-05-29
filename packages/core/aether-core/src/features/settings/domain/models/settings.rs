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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn should_use_default_values_when_created_via_default() {
        let settings = Settings::default();

        assert_eq!(
            settings.max_concurrent_downloads(),
            DEFAULT_MAX_CONCURRENT_DOWNLOADS
        );
        assert!(settings.enabled_plugins().is_empty());
    }

    #[test]
    fn should_return_constructed_values_when_created_via_new() {
        let mut plugins = HashSet::new();
        plugins.insert("plugin_a".to_owned());
        plugins.insert("plugin_b".to_owned());

        let settings = Settings::new(5, plugins.clone());

        assert_eq!(settings.max_concurrent_downloads(), 5);
        assert_eq!(settings.enabled_plugins(), &plugins);
    }

    #[test]
    fn should_update_max_concurrent_downloads_when_setter_called() {
        let mut settings = Settings::default();

        settings.set_max_concurrent_downloads(8);

        assert_eq!(settings.max_concurrent_downloads(), 8);
    }

    #[test]
    fn should_return_true_when_enabling_a_new_plugin() {
        let mut settings = Settings::default();

        let result = settings.enable_plugin("my_plugin");

        assert!(result);
        assert!(settings.is_plugin_enabled("my_plugin"));
    }

    #[test]
    fn should_return_false_when_enabling_an_already_enabled_plugin() {
        let mut settings = Settings::default();
        settings.enable_plugin("my_plugin");

        let result = settings.enable_plugin("my_plugin");

        assert!(!result);
    }

    #[test]
    fn should_return_true_when_disabling_an_enabled_plugin() {
        let mut settings = Settings::default();
        settings.enable_plugin("my_plugin");

        let result = settings.disable_plugin("my_plugin");

        assert!(result);
        assert!(!settings.is_plugin_enabled("my_plugin"));
    }

    #[test]
    fn should_return_false_when_disabling_a_not_enabled_plugin() {
        let mut settings = Settings::default();

        let result = settings.disable_plugin("nonexistent");

        assert!(!result);
    }

    #[test]
    fn should_check_plugin_enabled_correctly() {
        let mut settings = Settings::default();
        settings.enable_plugin("active_plugin");

        assert!(settings.is_plugin_enabled("active_plugin"));
        assert!(!settings.is_plugin_enabled("inactive_plugin"));
    }
}
