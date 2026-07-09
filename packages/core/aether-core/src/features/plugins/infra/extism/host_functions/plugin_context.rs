use std::sync::Arc;

use crate::core::app::AetherContainer;

#[derive(Clone)]
pub struct PluginContext {
    pub id: String,
    pub container: Arc<AetherContainer>,
}
