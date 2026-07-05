use std::sync::Arc;

use crate::features::plugins::{PluginError, PluginSettings, PluginSettingsStorage};

pub struct GetPluginSettingsUseCase {
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
}

impl GetPluginSettingsUseCase {
    pub fn new(plugin_settings_storage: Arc<dyn PluginSettingsStorage>) -> Self {
        Self {
            plugin_settings_storage,
        }
    }

    pub async fn execute(&self, plugin_id: String) -> Result<Option<PluginSettings>, PluginError> {
        self.plugin_settings_storage.get(&plugin_id).await
    }
}
