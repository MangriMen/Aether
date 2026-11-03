use std::collections::{HashMap, HashSet};

use super::MaxConcurrentDownloads;
use crate::features::instance::PlatformId;

#[derive(Debug, Clone, Default)]
pub struct Settings {
    max_concurrent_downloads: MaxConcurrentDownloads,

    enabled_plugins: HashSet<String>,

    /// Platform → id of the provider the user chose to serve it. Providers are stored as strings
    /// so the settings do not depend on the instance feature.
    platform_providers: HashMap<PlatformId, String>,
}

impl Settings {
    pub fn new(
        max_concurrent_downloads: MaxConcurrentDownloads,
        enabled_plugins: HashSet<String>,
    ) -> Self {
        Self {
            max_concurrent_downloads,
            enabled_plugins,
            platform_providers: HashMap::default(),
        }
    }

    #[must_use]
    pub fn with_platform_providers(
        mut self,
        platform_providers: HashMap<PlatformId, String>,
    ) -> Self {
        self.platform_providers = platform_providers;
        self
    }

    pub fn platform_providers(&self) -> &HashMap<PlatformId, String> {
        &self.platform_providers
    }

    /// Choose `provider_id` for `platform`, or drop the choice with `None`. Returns whether
    /// anything changed.
    pub fn set_platform_provider(
        &mut self,
        platform: &PlatformId,
        provider_id: Option<&str>,
    ) -> bool {
        match provider_id {
            Some(provider_id) => {
                self.platform_providers
                    .insert(platform.clone(), provider_id.to_owned())
                    .as_deref()
                    != Some(provider_id)
            }
            None => self.platform_providers.remove(platform).is_some(),
        }
    }

    pub fn max_concurrent_downloads(&self) -> MaxConcurrentDownloads {
        self.max_concurrent_downloads
    }

    pub fn enabled_plugins(&self) -> &HashSet<String> {
        &self.enabled_plugins
    }

    pub fn set_max_concurrent_downloads(
        &mut self,
        max_concurrent_downloads: MaxConcurrentDownloads,
    ) {
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

/// A platform that is not a [`PlatformId`] is dropped: a bad key must not fail the whole read.
pub(crate) fn parse_platform_providers(
    raw: impl IntoIterator<Item = (String, String)>,
) -> HashMap<PlatformId, String> {
    raw.into_iter()
        .filter_map(
            |(platform, provider)| match PlatformId::try_from(platform.as_str()) {
                Ok(platform) => Some((platform, provider)),
                Err(err) => {
                    tracing::warn!(
                        "Dropping the provider choice for platform \"{platform}\": {err}"
                    );
                    None
                }
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_report_a_changed_platform_provider_only_when_it_differs() {
        let mut settings = Settings::default();

        assert!(
            settings.set_platform_provider(&PlatformId::from_static("curseforge"), Some("a#b"))
        );
        assert!(
            !settings.set_platform_provider(&PlatformId::from_static("curseforge"), Some("a#b"))
        );
        assert!(
            settings.set_platform_provider(&PlatformId::from_static("curseforge"), Some("c#d"))
        );
        assert!(settings.set_platform_provider(&PlatformId::from_static("curseforge"), None));
        assert!(!settings.set_platform_provider(&PlatformId::from_static("curseforge"), None));
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
