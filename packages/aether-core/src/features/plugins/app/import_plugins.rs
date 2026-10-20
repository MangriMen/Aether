use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::features::plugins::{
    ExtractedPlugin, PluginError, PluginExtractor, PluginRegistry, PluginSettingsStorage,
    PluginSourceStorage, PluginStorage, PluginSyncService, PluginVerificationStorage,
};

use super::ports::ImportPluginsUseCasePort;

/// Installs plugins from local archives.
///
/// A local archive is nobody's release: a plugin imported over one installed from a provider
/// loses that provider's records with its old bytes — the sha256 recorded for them (the loader
/// would refuse the new wasm against it) and its source (an update would replace the local build
/// with the provider's). Its settings stay.
///
/// An installed plugin is replaced as a whole, as an update does, not merged with: files the
/// new archive does not have must not stay behind to run with the new build. The sync that
/// follows loads the new build of a plugin that was running.
pub(crate) struct ImportPluginsUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    plugin_extractor: Arc<dyn PluginExtractor>,
    plugin_storage: Arc<dyn PluginStorage>,
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    plugin_verification_storage: Arc<dyn PluginVerificationStorage>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
}

impl ImportPluginsUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
        plugin_extractor: Arc<dyn PluginExtractor>,
        plugin_storage: Arc<dyn PluginStorage>,
        plugin_source_storage: Arc<dyn PluginSourceStorage>,
        plugin_verification_storage: Arc<dyn PluginVerificationStorage>,
        sync_plugins_service: Arc<dyn PluginSyncService>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_settings_storage,
            plugin_extractor,
            plugin_storage,
            plugin_source_storage,
            plugin_verification_storage,
            sync_plugins_service,
        }
    }

    pub(crate) async fn execute(&self, paths: Vec<PathBuf>) -> Result<(), PluginError> {
        let mut import_result = Ok(());
        for path in paths {
            if let Err(e) = self.import_one(&path).await {
                import_result = Err(e);
                break;
            }
        }

        // Also after a failure: the archives imported before it are on disk and belong in the
        // registry.
        let sync_result = self.sync_plugins_service.execute().await;
        import_result?;
        sync_result
    }

    async fn import_one(&self, path: &std::path::Path) -> Result<(), PluginError> {
        let extracted_plugin = self.plugin_extractor.extract(path).await?;
        let plugin_id = extracted_plugin.plugin_id.clone();

        // Before the new bytes land, so they are never checked against the old record. Should
        // the copy fail, the old bytes are left unverified — loaded with a warning, not refused.
        let old_verification = self.plugin_verification_storage.get(&plugin_id).await?;
        let old_source = self.plugin_source_storage.get(&plugin_id).await?;
        self.plugin_verification_storage.remove(&plugin_id).await?;
        self.plugin_source_storage.remove(&plugin_id).await?;

        // The old bytes stay when the import fails, so their records must too.
        let result = self
            .replace_keeping_settings(&plugin_id, extracted_plugin)
            .await;
        if result.is_err() {
            if let Some(verification) = &old_verification
                && let Err(err) = self
                    .plugin_verification_storage
                    .save(&plugin_id, verification)
                    .await
            {
                tracing::error!(
                    "Verification of plugin '{plugin_id}' could not be restored: {err}"
                );
            }
            if let Some(source) = &old_source
                && let Err(err) = self.plugin_source_storage.save(&plugin_id, source).await
            {
                tracing::error!("Source of plugin '{plugin_id}' could not be restored: {err}");
            }
        }
        result
    }

    async fn replace_keeping_settings(
        &self,
        plugin_id: &str,
        extracted_plugin: ExtractedPlugin,
    ) -> Result<(), PluginError> {
        // The settings live in the plugin's directory, which the replacement swaps out. Held
        // until they are back, so that a change made meanwhile is neither saved over nor lost.
        let _settings_lock = self.plugin_registry.lock_settings(plugin_id).await;
        let settings = self.plugin_settings_storage.get(plugin_id).await?;

        self.plugin_storage.replace(extracted_plugin).await?;

        if let Some(settings) = settings
            && let Err(err) = self
                .plugin_settings_storage
                .upsert(plugin_id, &settings)
                .await
        {
            if let Err(restore_err) = self.plugin_storage.restore_replaced(plugin_id).await {
                tracing::error!(
                    "Plugin '{plugin_id}' could not be put back after a failed import: \
                     {restore_err}"
                );
            }
            return Err(err);
        }

        if let Err(err) = self.plugin_storage.discard_replaced(plugin_id).await {
            tracing::warn!("Previous version of plugin '{plugin_id}' was left behind: {err}");
        }
        Ok(())
    }
}

#[async_trait]
impl ImportPluginsUseCasePort for ImportPluginsUseCase {
    async fn execute(&self, paths: Vec<PathBuf>) -> Result<(), PluginError> {
        self.execute(paths).await
    }
}
