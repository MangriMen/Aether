use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{Plugin, PluginError, PluginRegistry, PluginSettingsStorage};

use super::PluginDto;
use super::get_plugin_dto::plugin_dto;
use super::ports::ListPluginsDtoUseCasePort;

pub(crate) struct ListPluginsDtoUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
}

impl ListPluginsDtoUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_settings_storage,
        }
    }

    pub(crate) async fn execute(&self) -> Result<Vec<PluginDto>, PluginError> {
        // Taken out of the registry first: no entry stays locked while settings are read.
        let plugins: Vec<Plugin> = self
            .plugin_registry
            .list()
            .map(|plugin| plugin.clone())
            .collect();

        let mut dtos = Vec::with_capacity(plugins.len());
        for plugin in plugins {
            dtos.push(plugin_dto(self.plugin_settings_storage.as_ref(), plugin).await);
        }

        Ok(dtos)
    }
}

#[async_trait]
impl ListPluginsDtoUseCasePort for ListPluginsDtoUseCase {
    async fn execute(&self) -> Result<Vec<PluginDto>, PluginError> {
        self.execute().await
    }
}
