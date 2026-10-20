use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginDisableService, PluginError, PluginStorage, PluginSyncService,
};

use super::ports::RemovePluginUseCasePort;

pub(crate) struct RemovePluginUseCase {
    plugin_storage: Arc<dyn PluginStorage>,
    disable_plugin_service: Arc<dyn PluginDisableService>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
}

impl RemovePluginUseCase {
    pub(crate) fn new(
        plugin_storage: Arc<dyn PluginStorage>,
        disable_plugin_service: Arc<dyn PluginDisableService>,
        sync_plugins_service: Arc<dyn PluginSyncService>,
    ) -> Self {
        Self {
            plugin_storage,
            disable_plugin_service,
            sync_plugins_service,
        }
    }

    pub(crate) async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        // Unload first: a busy plugin refuses with `Busy` while its files are still intact,
        // instead of leaving a registry entry whose directory is already gone.
        match self.disable_plugin_service.execute(plugin_id.clone()).await {
            Ok(()) | Err(PluginError::NotFound { .. } | PluginError::AlreadyUnloaded { .. }) => {}
            Err(err) => return Err(err),
        }

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
