use std::sync::Arc;

use crate::features::{
    events::EventEmitter,
    plugins::{PluginError, PluginRegistry},
};

use super::PluginDto;

pub struct ListPluginsDtoUseCase<E: EventEmitter> {
    plugin_registry: Arc<PluginRegistry<E>>,
}

impl<E: EventEmitter> ListPluginsDtoUseCase<E> {
    pub fn new(plugin_registry: Arc<PluginRegistry<E>>) -> Self {
        Self { plugin_registry }
    }

    pub async fn execute(&self) -> Result<Vec<PluginDto>, PluginError> {
        Ok(self.plugin_registry.list().map(PluginDto::from).collect())
    }
}
