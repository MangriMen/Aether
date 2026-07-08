use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginRegistry};

use super::PluginDto;
use super::ports::GetPluginDtoUseCasePort;

pub struct GetPluginDtoUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl GetPluginDtoUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }
    pub async fn execute(&self, plugin_id: String) -> Result<PluginDto, PluginError> {
        self.plugin_registry.get(&plugin_id).map(PluginDto::from)
    }
}

#[async_trait]
impl GetPluginDtoUseCasePort for GetPluginDtoUseCase {
    async fn execute(&self, plugin_id: String) -> Result<PluginDto, PluginError> {
        self.execute(plugin_id).await
    }
}
