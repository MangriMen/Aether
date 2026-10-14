use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginExtractor, PluginSourceStorage, PluginStorage, PluginSyncService,
    PluginVerificationStorage,
};

use super::ports::ImportPluginsUseCasePort;

/// Installs plugins from local archives.
///
/// A local archive is nobody's release: a plugin imported over one installed from a provider
/// loses that provider's records with its old bytes — the sha256 recorded for them (the loader
/// would refuse the new wasm against it) and its source (an update would replace the local build
/// with the provider's). Its settings stay.
pub(crate) struct ImportPluginsUseCase {
    plugin_extractor: Arc<dyn PluginExtractor>,
    plugin_storage: Arc<dyn PluginStorage>,
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    plugin_verification_storage: Arc<dyn PluginVerificationStorage>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
}

impl ImportPluginsUseCase {
    pub(crate) fn new(
        plugin_extractor: Arc<dyn PluginExtractor>,
        plugin_storage: Arc<dyn PluginStorage>,
        plugin_source_storage: Arc<dyn PluginSourceStorage>,
        plugin_verification_storage: Arc<dyn PluginVerificationStorage>,
        sync_plugins_service: Arc<dyn PluginSyncService>,
    ) -> Self {
        Self {
            plugin_extractor,
            plugin_storage,
            plugin_source_storage,
            plugin_verification_storage,
            sync_plugins_service,
        }
    }

    pub(crate) async fn execute(&self, paths: Vec<PathBuf>) -> Result<(), PluginError> {
        for path in paths {
            let extracted_plugin = self.plugin_extractor.extract(&path).await?;
            let plugin_id = extracted_plugin.plugin_id.clone();

            // Before the new bytes land, so they are never checked against the old record. Should
            // the copy fail, the old bytes are left unverified — loaded with a warning, not refused.
            self.plugin_verification_storage.remove(&plugin_id).await?;
            self.plugin_source_storage.remove(&plugin_id).await?;
            self.plugin_storage.add(extracted_plugin).await?;
        }

        self.sync_plugins_service.execute().await?;
        Ok(())
    }
}

#[async_trait]
impl ImportPluginsUseCasePort for ImportPluginsUseCase {
    async fn execute(&self, paths: Vec<PathBuf>) -> Result<(), PluginError> {
        self.execute(paths).await
    }
}
