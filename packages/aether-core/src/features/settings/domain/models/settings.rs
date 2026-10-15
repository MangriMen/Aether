use std::collections::{HashMap, HashSet};

pub const DEFAULT_MAX_CONCURRENT_DOWNLOADS: usize = 10;
#[allow(clippy::cast_possible_wrap)]
pub const DEFAULT_MAX_CONCURRENT_DOWNLOADS_I64: i64 = DEFAULT_MAX_CONCURRENT_DOWNLOADS as i64;

#[derive(Debug, Clone)]
pub struct Settings {
    max_concurrent_downloads: usize,

    enabled_plugins: HashSet<String>,

    /// Platform → id of the provider the user chose to serve it. Providers are stored as strings
    /// so the settings do not depend on the instance feature.
    platform_providers: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: DEFAULT_MAX_CONCURRENT_DOWNLOADS,
            enabled_plugins: HashSet::default(),
            platform_providers: HashMap::default(),
        }
    }
}

impl Settings {
    pub fn new(max_concurrent_downloads: usize, enabled_plugins: HashSet<String>) -> Self {
        Self {
            max_concurrent_downloads,
            enabled_plugins,
            platform_providers: HashMap::default(),
        }
    }

    #[must_use]
    pub fn with_platform_providers(mut self, platform_providers: HashMap<String, String>) -> Self {
        self.platform_providers = platform_providers;
        self
    }

    pub fn platform_providers(&self) -> &HashMap<String, String> {
        &self.platform_providers
    }

    /// Choose `provider_id` for `platform`, or drop the choice with `None`. Returns whether
    /// anything changed.
    pub fn set_platform_provider(&mut self, platform: &str, provider_id: Option<&str>) -> bool {
        match provider_id {
            Some(provider_id) => {
                self.platform_providers
                    .insert(platform.to_owned(), provider_id.to_owned())
                    .as_deref()
                    != Some(provider_id)
            }
            None => self.platform_providers.remove(platform).is_some(),
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
    use super::*;

    #[test]
    fn should_report_a_changed_platform_provider_only_when_it_differs() {
        let mut settings = Settings::default();

        assert!(settings.set_platform_provider("curseforge", Some("a#b")));
        assert!(!settings.set_platform_provider("curseforge", Some("a#b")));
        assert!(settings.set_platform_provider("curseforge", Some("c#d")));
        assert!(settings.set_platform_provider("curseforge", None));
        assert!(!settings.set_platform_provider("curseforge", None));
    }

    #[test]
    fn should_return_false_when_enabling_an_already_enabled_plugin() {
        let mut settings = Settings::default();
        settings.enable_plugin("my_plugin");

        let result = settings.enable_plugin("my_plugin");

        assert!(!result);
    }

    #[test]
    fn should_return_false_when_disabling_a_not_enabled_plugin() {
        let mut settings = Settings::default();

        let result = settings.disable_plugin("nonexistent");

        assert!(!result);
    }
}
