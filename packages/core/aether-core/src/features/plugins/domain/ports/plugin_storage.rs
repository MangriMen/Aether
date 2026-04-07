use std::collections::HashMap;

use async_trait::async_trait;

use crate::features::plugins::{ExtractedPlugin, Plugin, PluginError};

#[async_trait]
pub trait PluginStorage: Send + Sync {
    async fn add(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError>;
    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError>;
    async fn get(&self, plugin_id: &str) -> Result<Plugin, PluginError>;
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError>;
}
