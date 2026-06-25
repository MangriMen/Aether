use crate::features::plugins::domain::{
    PluginError, PluginSourceType, ProviderPluginPreview, ProviderUpdateInfo,
};

/// Common interface for all plugin providers (GitHub, custom server, Modrinth, etc.).
///
/// This is the Strategy pattern abstraction — each provider implements
/// its own way of fetching metadata, checking updates, and downloading plugins.
#[async_trait::async_trait]
pub trait PluginProvider: Send + Sync {
    /// Fetch preview metadata (manifest, capabilities, releases) for a plugin.
    async fn fetch_preview(&self, identifier: &str) -> Result<ProviderPluginPreview, PluginError>;

    /// Get the latest version info for update checking.
    async fn fetch_latest_version(
        &self,
        identifier: &str,
        current_tag: &str,
        current_version: &str,
    ) -> Result<ProviderUpdateInfo, PluginError>;

    /// Download the plugin archive bytes for a specific version.
    async fn download_plugin(&self, download_url: &str) -> Result<Vec<u8>, PluginError>;

    /// Return which source type this provider handles.
    fn source_type(&self) -> PluginSourceType;

    /// Parse a user-facing identifier (e.g., GitHub URL) into a normalized form.
    fn parse_identifier(&self, raw: &str) -> Result<String, PluginError>;
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

    /// Get all registered provider source types.
    pub fn list_source_types(&self) -> Vec<PluginSourceType> {
        self.providers.iter().map(|p| p.source_type()).collect()
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
