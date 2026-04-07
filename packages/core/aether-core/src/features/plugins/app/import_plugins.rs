use std::{path::PathBuf, sync::Arc};

use crate::features::{
    events::EventEmitter,
    plugins::{PluginError, PluginExtractor, PluginLoader, PluginStorage},
    settings::SettingsStorage,
};

use super::SyncPluginsUseCase;

pub struct ImportPluginsUseCase<
    PS: PluginStorage,
    SS: SettingsStorage,
    PL: PluginLoader,
    E: EventEmitter,
    PE: PluginExtractor,
> {
    plugin_extractor: Arc<PE>,
    plugin_storage: Arc<PS>,
    sync_plugins_use_case: Arc<SyncPluginsUseCase<PS, SS, PL, E>>,
}

impl<
        PS: PluginStorage,
        SS: SettingsStorage,
        PL: PluginLoader,
        E: EventEmitter,
        PE: PluginExtractor,
    > ImportPluginsUseCase<PS, SS, PL, E, PE>
{
    pub fn new(
        plugin_extractor: Arc<PE>,
        plugin_storage: Arc<PS>,
        sync_plugins_use_case: Arc<SyncPluginsUseCase<PS, SS, PL, E>>,
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
