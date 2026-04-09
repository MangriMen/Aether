use std::sync::Arc;

use crate::features::plugins::{PluginError, PluginRegistry};

use super::PluginDto;

pub struct ListPluginsDtoUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl ListPluginsDtoUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }

    pub async fn execute(&self) -> Result<Vec<PluginDto>, PluginError> {
        Ok(self.plugin_registry.list().map(PluginDto::from).collect())
    }
}
