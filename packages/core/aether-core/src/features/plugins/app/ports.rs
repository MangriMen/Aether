use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use semver;
use tokio::sync::Mutex;

use crate::features::plugins::app::{EditPluginSettings, PluginDto};
use crate::features::plugins::domain::PluginInstance;
use crate::features::plugins::{
    ExtractedPlugin, Plugin, PluginError, PluginManifest, PluginSettings, PluginSource,
    PluginVerification, ProviderUpdateInfo,
};

#[async_trait]
pub trait PluginSourceStorage: Send + Sync {
    async fn save(&self, plugin_id: &str, source: &PluginSource) -> Result<(), PluginError>;
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSource>, PluginError>;
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError>;
}

/// Persists the trust-on-first-use record established when a plugin is fetched
/// from a remote provider. Absence of a record means "never verified", which is
/// a valid state (locally installed plugins), not an error.
#[async_trait]
pub trait PluginVerificationStorage: Send + Sync {
    async fn save(
        &self,
        plugin_id: &str,
        verification: &PluginVerification,
    ) -> Result<(), PluginError>;
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginVerification>, PluginError>;
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
    /// Lowercase hex sha256 of the plugin's wasm blob as it currently sits on disk.
    async fn wasm_sha256(&self, plugin_id: &str) -> Result<String, PluginError>;
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

#[async_trait]
pub trait PluginSyncService: Send + Sync {
    async fn execute(&self) -> Result<(), PluginError>;
}

#[async_trait]
pub trait PluginDisableService: Send + Sync {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError>;
}

// ── Use case ports ──

#[async_trait]
pub trait CheckForPluginUpdatesUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: &str) -> Result<ProviderUpdateInfo, PluginError>;
}

#[async_trait]
pub trait EditPluginSettingsUseCasePort: Send + Sync {
    async fn execute(
        &self,
        plugin_id: String,
        edit_settings: EditPluginSettings,
    ) -> Result<(), PluginError>;
}

#[async_trait]
pub trait EnablePluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError>;
}

#[async_trait]
pub trait ForceEnablePluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError>;
}

#[async_trait]
pub trait GetPluginApiVersionUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<semver::Version, PluginError>;
}

#[async_trait]
pub trait GetPluginDtoUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: String) -> Result<PluginDto, PluginError>;
}

#[async_trait]
pub trait GetPluginSettingsUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: String) -> Result<Option<PluginSettings>, PluginError>;
}

#[async_trait]
pub trait ImportPluginsUseCasePort: Send + Sync {
    async fn execute(&self, paths: Vec<PathBuf>) -> Result<(), PluginError>;
}

#[async_trait]
pub trait ListPluginsDtoUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<PluginDto>, PluginError>;
}

#[async_trait]
pub trait RemovePluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError>;
}

#[async_trait]
pub trait UpdatePluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: &str, target_tag: Option<&str>) -> Result<(), PluginError>;
}
