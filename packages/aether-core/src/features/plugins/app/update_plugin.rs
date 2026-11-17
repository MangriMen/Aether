use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{
    PluginError, PluginExtractor, PluginRegistry, PluginSettingsStorage, PluginSource,
    PluginSourceStorage, PluginStorage, PluginSyncService, PluginVerificationStorage,
};

use super::ports::UpdatePluginUseCasePort;
use super::release_archive::{
    ReleaseSource, download_release_archive, ensure_previewed_archive, recorded_source,
};
use super::release_installer::ReleaseInstaller;
use super::{PluginProviderFactory, ReleaseArchiveCache};

/// Updates a plugin to the release the user previewed, provided its archive hashes to what the
/// preview showed: the user agreed to what that archive asks for, not to whatever the release
/// holds by the time of the update. Works with any provider type.
///
/// The new release replaces the plugin's files, and the user's settings of it carry over — all
/// but a force-enable, which confirms one specific build. An update that fails at any
/// step leaves the installed version, its settings, trust record and source as they were.
///
/// Once replaced, the plugin is synced, so that the registry holds the new build and a plugin
/// that was enabled runs it. The update is done by then, so a failed sync is logged, not returned.
pub(crate) struct UpdatePluginUseCase {
    plugin_extractor: Arc<dyn PluginExtractor>,
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    installer: ReleaseInstaller,
    provider_factory: Arc<PluginProviderFactory>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
    archive_cache: Arc<ReleaseArchiveCache>,
}

impl UpdatePluginUseCase {
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
            plugin_source_storage: plugin_source_storage.clone(),
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
        plugin_id: &PluginId,
        tag: &str,
        expected_sha256: &str,
    ) -> Result<(), PluginError> {
        let (source_type, identifier) =
            recorded_source(self.plugin_source_storage.as_ref(), plugin_id).await?;

        let downloaded = download_release_archive(
            &self.provider_factory,
            &self.archive_cache,
            &source_type,
            ReleaseSource::Recorded(&identifier),
            tag,
        )
        .await?;

        ensure_previewed_archive(&downloaded.archive, expected_sha256).await?;

        // Unpacked before anything is removed: an archive that cannot be read, or that is of
        // another plugin, leaves the installed version in place.
        let extracted = self
            .plugin_extractor
            .extract(downloaded.archive.path())
            .await?;
        if &extracted.plugin_id != plugin_id {
            return Err(PluginError::IdMismatch {
                plugin_id: plugin_id.clone(),
                archive_id: extracted.plugin_id,
            });
        }

        let updated_source = PluginSource::Remote {
            source_type,
            identifier: downloaded.identifier,
            current_tag: downloaded.release.tag_name,
            current_version: downloaded.release.version,
        };

        self.installer
            .install(extracted, &updated_source, true)
            .await?;

        // Once replaced, so that enabling the new build, which can take seconds, holds nothing.
        if let Err(err) = self.sync_plugins_service.execute().await {
            tracing::warn!("Plugin '{plugin_id}' was updated but the sync after it failed: {err}");
        }
        Ok(())
    }
}

#[async_trait]
impl UpdatePluginUseCasePort for UpdatePluginUseCase {
    async fn execute(
        &self,
        plugin_id: &PluginId,
        tag: &str,
        expected_sha256: &str,
    ) -> Result<(), PluginError> {
        self.execute(plugin_id, tag, expected_sha256).await
    }
}
