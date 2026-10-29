use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginDisableService, PluginError, PluginRegistry, PluginStorage, PluginSyncService,
};

use super::ports::RemovePluginUseCasePort;

pub(crate) struct RemovePluginUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_storage: Arc<dyn PluginStorage>,
    disable_plugin_service: Arc<dyn PluginDisableService>,
    sync_plugins_service: Arc<dyn PluginSyncService>,
}

impl RemovePluginUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_storage: Arc<dyn PluginStorage>,
        disable_plugin_service: Arc<dyn PluginDisableService>,
        sync_plugins_service: Arc<dyn PluginSyncService>,
    ) -> Self {
        Self {
            plugin_registry,
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

        // An enable that got in after the unload is refused here, with the files intact; from
        // here on none can load the plugin until it is gone. A plugin the registry does not
        // hold (its directory only) has nothing to guard.
        let previous = match self.plugin_registry.begin_removing(&plugin_id).await {
            Ok(previous) => Some(previous),
            Err(PluginError::NotFound { .. }) => None,
            Err(err) => return Err(err),
        };

        if let Err(err) = self.plugin_storage.remove(&plugin_id).await {
            if let Some(previous) = previous {
                self.plugin_registry
                    .abandon_removing(&plugin_id, previous)
                    .await?;
            }
            return Err(err);
        }

        self.plugin_registry.remove(&plugin_id);
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
