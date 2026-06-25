use std::sync::Arc;

use crate::features::plugins::{
    PluginError, PluginSource, PluginSourceStorage, ProviderUpdateInfo,
};

use super::PluginProviderFactory;

/// Checks for plugin updates using the provider factory.
/// Works with any provider type (GitHub, Modrinth, etc.).
pub struct CheckForPluginUpdatesUseCase<Src: PluginSourceStorage> {
    plugin_source_storage: Arc<Src>,
    provider_factory: Arc<PluginProviderFactory>,
}

impl<Src: PluginSourceStorage> CheckForPluginUpdatesUseCase<Src> {
    pub fn new(
        plugin_source_storage: Arc<Src>,
        provider_factory: Arc<PluginProviderFactory>,
    ) -> Self {
        Self {
            plugin_source_storage,
            provider_factory,
        }
    }

    /// Check if a plugin has updates available.
    pub async fn execute(&self, plugin_id: &str) -> Result<ProviderUpdateInfo, PluginError> {
        let source = self
            .plugin_source_storage
            .get(plugin_id)
            .await?
            .ok_or_else(|| PluginError::NotAGitHubPlugin {
                plugin_id: plugin_id.to_string(),
            })?;

        let source_type = source.to_source_type();
        let identifier = source_identifier(&source);

        let provider = self
            .provider_factory
            .get_provider(&source_type)
            .ok_or_else(|| PluginError::NotAGitHubPlugin {
                plugin_id: plugin_id.to_string(),
            })?;

        let current_tag = source.current_tag().unwrap_or_default();
        // We don't track current_version separately in the provider trait;
        // fetch the latest version info
        provider
            .fetch_latest_version(&identifier, current_tag, "")
            .await
    }
}

/// Build a provider identifier from a `PluginSource`.
fn source_identifier(source: &PluginSource) -> String {
    match source {
        PluginSource::GitHub { owner, repo, .. } => format!("{owner}/{repo}"),
        PluginSource::Local => String::new(),
    }
}
