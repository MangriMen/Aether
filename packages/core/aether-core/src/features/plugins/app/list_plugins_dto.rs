use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginRegistry};

use super::PluginDto;
use super::ports::ListPluginsDtoUseCasePort;

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

#[async_trait]
impl ListPluginsDtoUseCasePort for ListPluginsDtoUseCase {
    async fn execute(&self) -> Result<Vec<PluginDto>, PluginError> {
        self.execute().await
    }
}
