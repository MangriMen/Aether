use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{
    FolderMounts, Plugin, PluginError, PluginRegistry, PluginSettingsStorage, PluginState,
};

use super::PluginDto;
use super::ports::GetPluginDtoUseCasePort;

pub(crate) struct GetPluginDtoUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
}

impl GetPluginDtoUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_settings_storage,
        }
    }

    pub(crate) async fn execute(&self, plugin_id: PluginId) -> Result<PluginDto, PluginError> {
        let plugin = self.plugin_registry.get(&plugin_id)?.clone();
        Ok(plugin_dto(self.plugin_settings_storage.as_ref(), plugin).await)
    }
}

/// A running plugin needs a restart when the folders in its settings, as saved now, are not the
/// ones its instance was built with. Settings that cannot be read do not keep the plugin out of
/// the list: it is shown as needing no restart, and the error is left to whatever reads them next.
pub(super) async fn plugin_dto(storage: &dyn PluginSettingsStorage, plugin: Plugin) -> PluginDto {
    let plugin_id = &plugin.manifest.metadata.id;
    let restart_needed = match &plugin.state {
        PluginState::Loaded(handle) => match storage.get(plugin_id).await {
            Ok(settings) => {
                let saved = settings.as_ref().map_or(&[][..], |s| &s.allowed_paths);
                *handle.saved_folders() != FolderMounts::of(saved)
            }
            Err(err) => {
                tracing::warn!("Could not read the settings of plugin {plugin_id}: {err}");
                false
            }
        },
        _ => false,
    };

    PluginDto::new(plugin, restart_needed)
}

#[async_trait]
impl GetPluginDtoUseCasePort for GetPluginDtoUseCase {
    async fn execute(&self, plugin_id: PluginId) -> Result<PluginDto, PluginError> {
        self.execute(plugin_id).await
    }
}
