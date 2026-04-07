use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginSettings};

#[async_trait]
pub trait PluginSettingsStorage: Send + Sync {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, PluginError>;
    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), PluginError>;
}
