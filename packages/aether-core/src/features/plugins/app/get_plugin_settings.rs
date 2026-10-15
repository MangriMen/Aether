use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginSettings, PluginSettingsStorage};

use super::{edit_plugin_settings::settings_to_edit, ports::GetPluginSettingsUseCasePort};

pub(crate) struct GetPluginSettingsUseCase {
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
}

impl GetPluginSettingsUseCase {
    pub(crate) fn new(plugin_settings_storage: Arc<dyn PluginSettingsStorage>) -> Self {
        Self {
            plugin_settings_storage,
        }
    }

    pub(crate) async fn execute(
        &self,
        plugin_id: String,
    ) -> Result<Option<PluginSettings>, PluginError> {
        settings_to_edit(self.plugin_settings_storage.as_ref(), &plugin_id).await
    }
}

#[async_trait]
impl GetPluginSettingsUseCasePort for GetPluginSettingsUseCase {
    async fn execute(&self, plugin_id: String) -> Result<Option<PluginSettings>, PluginError> {
        self.execute(plugin_id).await
    }
}
