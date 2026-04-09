use std::sync::Arc;

use crate::features::{
    plugins::{PluginError, PluginLoader, PluginStorage},
    settings::SettingsStorage,
};

use super::SyncPluginsUseCase;

pub struct RemovePluginUseCase<PS: PluginStorage, SS: SettingsStorage, PL: PluginLoader> {
    plugin_storage: Arc<PS>,
    sync_plugins_use_case: Arc<SyncPluginsUseCase<PS, SS, PL>>,
}

impl<PS: PluginStorage, SS: SettingsStorage, PL: PluginLoader> RemovePluginUseCase<PS, SS, PL> {
    pub fn new(
        plugin_storage: Arc<PS>,

        sync_plugins_use_case: Arc<SyncPluginsUseCase<PS, SS, PL>>,
    ) -> Self {
        Self {
            plugin_storage,
            sync_plugins_use_case,
        }
    }

    pub async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        self.plugin_storage.remove(&plugin_id).await?;
        self.sync_plugins_use_case.execute().await?;
        Ok(())
    }
}
