use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginSource, PluginSourceStorage, ProviderUpdateInfo,
};

use super::PluginProviderFactory;
use super::ports::CheckForPluginUpdatesUseCasePort;

/// Checks for plugin updates using the provider factory.
/// Works with any provider type (GitHub, Modrinth, etc.).
pub struct CheckForPluginUpdatesUseCase {
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    provider_factory: Arc<PluginProviderFactory>,
}

impl CheckForPluginUpdatesUseCase {
    pub fn new(
        plugin_source_storage: Arc<dyn PluginSourceStorage>,
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
            .ok_or_else(|| PluginError::NoRemoteSource {
                plugin_id: plugin_id.to_string(),
            })?;

        let source_type = source.to_source_type();
        let identifier = source_identifier(&source);

        let provider = self
            .provider_factory
            .get_provider(&source_type)
            .ok_or_else(|| PluginError::NoRemoteSource {
                plugin_id: plugin_id.to_string(),
            })?;

        let current_tag = source.current_tag().unwrap_or_default();
        let current_version = source_version(&source);
        provider
            .fetch_latest_version(&identifier, current_tag, current_version)
            .await
    }
}

/// Build a provider identifier from a `PluginSource`.
fn source_identifier(source: &PluginSource) -> String {
    match source {
        PluginSource::Remote { identifier, .. } => identifier.clone(),
        PluginSource::Local => String::new(),
    }
}

fn source_version(source: &PluginSource) -> &str {
    match source {
        PluginSource::Remote {
            current_version, ..
        } => current_version,
        PluginSource::Local => "",
    }
}

#[async_trait]
impl CheckForPluginUpdatesUseCasePort for CheckForPluginUpdatesUseCase {
    async fn execute(&self, plugin_id: &str) -> Result<ProviderUpdateInfo, PluginError> {
        self.execute(plugin_id).await
    }
}
