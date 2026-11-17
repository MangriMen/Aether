use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{
    PluginError, PluginExtractor, PluginRegistry, PluginSettingsStorage, PluginSource,
    PluginSourceStorage, PluginSourceType, PluginStorage, PluginSyncService,
    PluginVerificationStorage,
};

use super::ports::InstallPluginFromProviderUseCasePort;
use super::release_archive::{ReleaseSource, download_release_archive, ensure_previewed_archive};
use super::release_installer::ReleaseInstaller;

use super::{PluginProviderFactory, ReleaseArchiveCache};

/// Installs the release a user picked, provided its archive hashes to what the preview showed.
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
    archive_cache: Arc<ReleaseArchiveCache>,
}

impl InstallPluginFromProviderUseCase {
    // One port per collaborator, wired once in the DI container; a parameter struct would only
    // repeat the field list.
    #[expect(
        clippy::too_many_arguments,
        reason = "the use case is built from its ports"
    )]
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_extractor: Arc<dyn PluginExtractor>,
        plugin_storage: Arc<dyn PluginStorage>,
        plugin_source_storage: Arc<dyn PluginSourceStorage>,
        plugin_verification_storage: Arc<dyn PluginVerificationStorage>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
        provider_factory: Arc<PluginProviderFactory>,
        sync_plugins_service: Arc<dyn PluginSyncService>,
        archive_cache: Arc<ReleaseArchiveCache>,
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
            archive_cache,
        }
    }

    pub(crate) async fn execute(
        &self,
        source_type: PluginSourceType,
        identifier: &str,
        tag: &str,
        expected_sha256: &str,
    ) -> Result<PluginId, PluginError> {
        let downloaded = download_release_archive(
            &self.provider_factory,
            &self.archive_cache,
            &source_type,
            ReleaseSource::Entered(identifier),
            tag,
        )
        .await?;
        let release = downloaded.release;
        let identifier = downloaded.identifier;

        ensure_previewed_archive(&downloaded.archive, expected_sha256).await?;

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
