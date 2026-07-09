use std::sync::{Arc, Weak};

use crate::core::app::AetherContainer;

#[derive(Clone)]
pub struct PluginContext {
    pub id: String,
    pub container: Weak<AetherContainer>,
}

impl PluginContext {
    pub fn new(id: String, container: &Arc<AetherContainer>) -> Self {
        Self {
            id,
            container: Arc::downgrade(container),
        }
    }

    pub fn upgrade_container(&self) -> Option<Arc<AetherContainer>> {
        self.container.upgrade()
    }
}
