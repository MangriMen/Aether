use std::sync::Arc;

use crate::features::{
    events::EventEmitter,
    plugins::{PluginError, PluginRegistry},
};

use super::PluginDto;

pub struct GetPluginDtoUseCase<E: EventEmitter> {
    plugin_registry: Arc<PluginRegistry<E>>,
}

impl<E: EventEmitter> GetPluginDtoUseCase<E> {
    pub fn new(plugin_registry: Arc<PluginRegistry<E>>) -> Self {
        Self { plugin_registry }
    }
    pub async fn execute(&self, plugin_id: String) -> Result<PluginDto, PluginError> {
        self.plugin_registry.get(&plugin_id).map(PluginDto::from)
    }
}
