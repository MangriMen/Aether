use std::sync::Arc;

use crate::features::{
    events::EventEmitter,
    plugins::{PluginError, PluginLoader, PluginStorage},
    settings::SettingsStorage,
};

use super::SyncPluginsUseCase;

pub struct RemovePluginUseCase<
    PS: PluginStorage,
    SS: SettingsStorage,
    PL: PluginLoader,
    E: EventEmitter,
> {
    plugin_storage: Arc<PS>,
    sync_plugins_use_case: Arc<SyncPluginsUseCase<PS, SS, PL, E>>,
}

impl<PS: PluginStorage, SS: SettingsStorage, PL: PluginLoader, E: EventEmitter>
    RemovePluginUseCase<PS, SS, PL, E>
{
    pub fn new(
        plugin_storage: Arc<PS>,

        sync_plugins_use_case: Arc<SyncPluginsUseCase<PS, SS, PL, E>>,
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
