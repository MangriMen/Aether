use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginStorage, PluginSyncService};

use super::ports::RemovePluginUseCasePort;

pub struct RemovePluginUseCase {
    plugin_storage: Arc<dyn PluginStorage>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
}

impl RemovePluginUseCase {
    pub fn new(
        plugin_storage: Arc<dyn PluginStorage>,

        sync_plugins_service: Arc<dyn PluginSyncService>,
    ) -> Self {
        Self {
            plugin_storage,
            sync_plugins_service,
        }
    }

    pub async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        self.plugin_storage.remove(&plugin_id).await?;
        self.sync_plugins_service.execute().await?;
        Ok(())
    }
}

#[async_trait]
impl RemovePluginUseCasePort for RemovePluginUseCase {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        self.execute(plugin_id).await
    }
}
