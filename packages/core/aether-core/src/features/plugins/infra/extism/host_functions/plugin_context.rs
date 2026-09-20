use std::sync::{Arc, Weak};

use crate::core::app::AetherContainer;

#[derive(Clone)]
pub struct PluginContext {
    pub id: String,
    pub container: Weak<AetherContainer>,
    /// Hosts the plugin may reach, already merged from the manifest and the user's
    /// plugin settings. Host functions that touch the network re-check against this
    /// list themselves instead of trusting Extism's own check (see `features::http`).
    pub allowed_hosts: Vec<String>,
}

impl PluginContext {
    pub fn new(id: String, container: &Arc<AetherContainer>, allowed_hosts: Vec<String>) -> Self {
        Self {
            id,
            container: Arc::downgrade(container),
            allowed_hosts,
        }
    }

    pub fn upgrade_container(&self) -> Option<Arc<AetherContainer>> {
        self.container.upgrade()
    }
}
