use std::{path::PathBuf, sync::Arc};

use crate::features::plugins::{PluginError, PluginExtractor, PluginStorage};

use super::SyncPluginsUseCase;

pub struct ImportPluginsUseCase {
    plugin_extractor: Arc<dyn PluginExtractor>,
    plugin_storage: Arc<dyn PluginStorage>,
    sync_plugins_use_case: Arc<SyncPluginsUseCase>,
}

impl ImportPluginsUseCase {
    pub fn new(
        plugin_extractor: Arc<dyn PluginExtractor>,
        plugin_storage: Arc<dyn PluginStorage>,
        sync_plugins_use_case: Arc<SyncPluginsUseCase>,
    ) -> Self {
        Self {
            plugin_extractor,
            plugin_storage,
            sync_plugins_use_case,
        }
    }

    pub async fn execute(&self, paths: Vec<PathBuf>) -> Result<(), PluginError> {
        for path in paths {
            let extracted_plugin = self.plugin_extractor.extract(&path).await?;
            self.plugin_storage.add(extracted_plugin).await?;
        }

        self.sync_plugins_use_case.execute().await?;
        Ok(())
    }
}
