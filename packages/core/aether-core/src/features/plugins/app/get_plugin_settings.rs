use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginSettings, PluginSettingsStorage};

use super::ports::GetPluginSettingsUseCasePort;

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

#[async_trait]
impl GetPluginSettingsUseCasePort for GetPluginSettingsUseCase {
    async fn execute(&self, plugin_id: String) -> Result<Option<PluginSettings>, PluginError> {
        self.execute(plugin_id).await
    }
}
