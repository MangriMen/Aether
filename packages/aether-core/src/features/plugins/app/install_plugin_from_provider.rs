use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{
    PluginError, PluginExtractor, PluginProviderError, PluginRegistry, PluginSettingsStorage,
    PluginSource, PluginSourceStorage, PluginSourceType, PluginStorage, PluginSyncService,
    PluginVerificationStorage,
};

use super::PluginProviderFactory;
use super::ports::InstallPluginFromProviderUseCasePort;
use super::release_archive::download_release_archive;
use super::release_installer::ReleaseInstaller;

/// Installs the release a user picked from a provider, if its archive is the one that was
/// previewed: it must hash to what the preview gave.
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
        tag: &str,
        expected_sha256: &str,
    ) -> Result<PluginId, PluginError> {
        let provider = self
            .provider_factory
            .get_provider(&source_type)
            .ok_or_else(|| PluginProviderError::ProviderFetchError {
                source_type: source_type.clone(),
                details: "No provider is registered for it".to_string(),
            })?;
        let identifier = provider.parse_identifier(identifier)?;

        let downloaded =
            download_release_archive(provider, &source_type, &identifier, Some(tag)).await?;
        let release = downloaded.release;

        // Refused before the archive is read: it has to be the one that was previewed.
        if !downloaded
            .archive
            .sha256()
            .eq_ignore_ascii_case(expected_sha256.trim())
        {
            return Err(PluginProviderError::ArchiveChangedSincePreview.into());
        }

        let extracted = self
            .plugin_extractor
            .extract(downloaded.archive.path())
            .await?;
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
        tag: &str,
        expected_sha256: &str,
    ) -> Result<PluginId, PluginError> {
        self.execute(source_type, identifier, tag, expected_sha256)
            .await
    }
}
