use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::features::plugins::domain::PluginInstance;
use crate::features::plugins::{
    ExtractedPlugin, Plugin, PluginError, PluginManifest, PluginSettings, PluginSource,
};

#[async_trait]
pub trait PluginSourceStorage: Send + Sync {
    async fn save(&self, plugin_id: &str, source: &PluginSource) -> Result<(), PluginError>;
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSource>, PluginError>;
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError>;
}

#[async_trait]
pub trait PluginLoader: Send + Sync {
    async fn load(
        &self,
        plugin: &PluginManifest,
        settings: Option<&PluginSettings>,
    ) -> Result<Arc<Mutex<dyn PluginInstance>>, PluginError>;
    async fn unload(&self, instance: Arc<Mutex<dyn PluginInstance>>) -> Result<(), PluginError>;
}

#[async_trait]
pub trait PluginStorage: Send + Sync {
    async fn add(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError>;
    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError>;
    async fn get(&self, plugin_id: &str) -> Result<Plugin, PluginError>;
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError>;
}

#[async_trait]
pub trait PluginSettingsStorage: Send + Sync {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, PluginError>;
    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), PluginError>;
}

#[async_trait]
pub trait PluginExtractor: Send + Sync {
    async fn extract(&self, file_path: &Path) -> Result<ExtractedPlugin, PluginError>;
}

pub trait AsCapabilityMetadata {
    fn as_metadata(&self) -> &crate::features::instance::CapabilityMetadata;
}
