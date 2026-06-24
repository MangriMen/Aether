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
    /// The plugin's API version requirement is incompatible with the host version.
    /// Contains a human-readable explanation (e.g. "requires ^3.0.0, host is 2.5.0").
    Incompatible(String),
    Failed(String),
}

#[derive(Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub capabilities: Option<PluginCapabilities>,
    pub hash: String,
    pub state: PluginState,
}
