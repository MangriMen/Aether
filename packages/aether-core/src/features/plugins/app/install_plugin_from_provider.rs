use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginExtractor, PluginProviderError, PluginRegistry, PluginSettingsStorage,
    PluginSource, PluginSourceStorage, PluginSourceType, PluginStorage, PluginSyncService,
    PluginVerificationStorage,
};

use super::ports::{InstallPluginFromProviderUseCasePort, ProviderRelease};
use super::release_installer::ReleaseInstaller;
use super::{PluginProviderFactory, write_bytes_to_temp_file};

/// Installs the release a user picked from a provider.
///
/// The archive is unpacked before anything is written, and the files and the records about them
/// (trust on first use, source, settings) go in as a whole: an install that fails at any step
/// leaves the plugin as it was, installed or not. A plugin that is installed already is replaced
/// as an update does, keeping its settings.
///
/// Once installed, the plugin is synced, so that the registry holds it. The install is done by
/// then, so a failed sync is logged, not returned.
pub(crate) struct InstallPluginFromProviderUseCase {
    plugin_extractor: Arc<dyn PluginExtractor>,
    installer: ReleaseInstaller,
    provider_factory: Arc<PluginProviderFactory>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
}

impl InstallPluginFromProviderUseCase {
    // One port per collaborator, wired once in the DI container; a parameter struct would only
    // repeat the field list.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_extractor: Arc<dyn PluginExtractor>,
        plugin_storage: Arc<dyn PluginStorage>,
        plugin_source_storage: Arc<dyn PluginSourceStorage>,
        plugin_verification_storage: Arc<dyn PluginVerificationStorage>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
        provider_factory: Arc<PluginProviderFactory>,
        sync_plugins_service: Arc<dyn PluginSyncService>,
    ) -> Self {
        Self {
            plugin_extractor,
            installer: ReleaseInstaller {
                registry: plugin_registry,
                storage: plugin_storage,
                source_storage: plugin_source_storage,
                verification_storage: plugin_verification_storage,
                settings_storage: plugin_settings_storage,
            },
            provider_factory,
            sync_plugins_service,
        }
    }

    pub(crate) async fn execute(
        &self,
        source_type: PluginSourceType,
        identifier: &str,
        release: ProviderRelease,
    ) -> Result<String, PluginError> {
        let provider = self
            .provider_factory
            .get_provider(&source_type)
            .ok_or_else(|| PluginProviderError::ProviderFetchError {
                source_type: source_type.clone(),
                details: "No provider is registered for it".to_string(),
            })?;
        let identifier = provider.parse_identifier(identifier)?;

        let zip_bytes = provider.download_plugin(&release.download_url).await?;
        let temp_file = write_bytes_to_temp_file(&zip_bytes)?;
        let extracted = self.plugin_extractor.extract(temp_file.path()).await?;
        let plugin_id = extracted.plugin_id.clone();

        let source = PluginSource::Remote {
            source_type,
            identifier,
            current_tag: release.tag_name,
            current_version: release.version,
        };
        self.installer.install(extracted, &source, false).await?;

        if let Err(err) = self.sync_plugins_service.execute().await {
            tracing::warn!(
                "Plugin '{plugin_id}' was installed but the sync after it failed: {err}"
            );
        }
        Ok(plugin_id)
    }
}

#[async_trait]
impl InstallPluginFromProviderUseCasePort for InstallPluginFromProviderUseCase {
    async fn execute(
        &self,
        source_type: PluginSourceType,
        identifier: &str,
        release: ProviderRelease,
    ) -> Result<String, PluginError> {
        self.execute(source_type, identifier, release).await
    }
}
