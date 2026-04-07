use std::sync::Arc;
use tokio::sync::Mutex;

use crate::features::plugins::{PluginCapabilities, PluginInstance};

use super::PluginManifest;

#[derive(Clone)]
pub enum PluginState {
    NotLoaded,
    Loading,
    Loaded(Arc<Mutex<dyn PluginInstance>>),
    Unloading,
    Failed(String),
}

#[derive(Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub capabilities: Option<PluginCapabilities>,
    pub hash: String,
    pub state: PluginState,
}
