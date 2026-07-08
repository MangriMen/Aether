use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginExtractor, PluginStorage, PluginSyncService};

use super::ports::ImportPluginsUseCasePort;

pub struct ImportPluginsUseCase {
    plugin_extractor: Arc<dyn PluginExtractor>,
    plugin_storage: Arc<dyn PluginStorage>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
}

impl ImportPluginsUseCase {
    pub fn new(
        plugin_extractor: Arc<dyn PluginExtractor>,
        plugin_storage: Arc<dyn PluginStorage>,
        sync_plugins_service: Arc<dyn PluginSyncService>,
    ) -> Self {
        Self {
            plugin_extractor,
            plugin_storage,
            sync_plugins_service,
        }
    }

    pub async fn execute(&self, paths: Vec<PathBuf>) -> Result<(), PluginError> {
        for path in paths {
            let extracted_plugin = self.plugin_extractor.extract(&path).await?;
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
