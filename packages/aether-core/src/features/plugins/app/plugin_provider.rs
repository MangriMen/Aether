use crate::features::plugins::domain::{
    PluginProviderError, PluginProviderInfo, PluginSourceType, ProviderPluginPreview,
    ProviderUpdateInfo,
};

/// Common interface for all plugin providers (GitHub, custom server, Modrinth, etc.).
///
/// This is the Strategy pattern abstraction — each provider implements
/// its own way of fetching metadata, checking updates, and downloading plugins.
#[async_trait::async_trait]
pub trait PluginProvider: Send + Sync {
    /// Fetch preview metadata (manifest, capabilities, releases) for a plugin.
    async fn fetch_preview(
        &self,
        identifier: &str,
    ) -> Result<ProviderPluginPreview, PluginProviderError>;

    /// Get the latest version info for update checking.
    async fn fetch_latest_version(
        &self,
        identifier: &str,
        current_tag: &str,
        current_version: &str,
    ) -> Result<ProviderUpdateInfo, PluginProviderError>;

    /// Download the plugin archive for a specific version into `dest`, returning the number of
    /// bytes written. The archive is not held in memory: the provider writes it as it arrives and
    /// refuses one that grows past its limit. On failure `dest` may hold a part of it; the caller
    /// owns the file and discards it.
    async fn download_plugin(
        &self,
        download_url: &str,
        dest: &mut tokio::fs::File,
    ) -> Result<u64, PluginProviderError>;

    /// Return which source type this provider handles.
    fn source_type(&self) -> PluginSourceType;

    /// The name the user sees for this provider. It is the provider's own, a proper name that
    /// is shown as it is and not translated.
    fn display_name(&self) -> String;

    /// Parse a user-facing identifier (e.g., GitHub URL) into a normalized form.
    fn parse_identifier(&self, raw: &str) -> Result<String, PluginProviderError>;
}

/// Factory that creates the appropriate `PluginProvider` based on source type.
///
/// New providers are registered here by adding a match arm.
pub struct PluginProviderFactory {
    providers: Vec<Box<dyn PluginProvider>>,
}

impl PluginProviderFactory {
    pub fn new(providers: Vec<Box<dyn PluginProvider>>) -> Self {
        Self { providers }
    }

    /// Get all registered providers, with the name each gives itself.
    pub fn list_providers(&self) -> Vec<PluginProviderInfo> {
        self.providers
            .iter()
            .map(|p| PluginProviderInfo {
                source_type: p.source_type(),
                name: p.display_name(),
            })
            .collect()
    }

    /// Get a provider for the given source type.
    pub fn get_provider(&self, source_type: &PluginSourceType) -> Option<&dyn PluginProvider> {
        self.providers
            .iter()
            .find(|p| p.source_type() == *source_type)
            .map(std::convert::AsRef::as_ref)
    }

    /// Get a provider that can parse the given identifier string.
    pub fn get_provider_for_identifier(&self, raw: &str) -> Option<&dyn PluginProvider> {
        self.providers
            .iter()
            .find_map(|p| p.parse_identifier(raw).ok().map(|_| p.as_ref()))
    }
}
