use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use semver;

use crate::features::plugins::app::{EditPluginSettings, FolderAccess, HostAccess, PluginDto};
use crate::features::plugins::domain::PluginHandle;
use crate::features::plugins::{
    ExtractedPlugin, Plugin, PluginError, PluginManifest, PluginProviderError, PluginSettings,
    PluginSource, PluginVerification, ProviderUpdateInfo,
};

#[async_trait]
pub trait PluginSourceStorage: Send + Sync {
    async fn save(&self, plugin_id: &str, source: &PluginSource) -> Result<(), PluginError>;
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSource>, PluginError>;
    /// No record is not an error.
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
    /// Forget the record: the bytes on disk are no longer the ones it vouched for. No record
    /// is not an error.
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError>;
}

#[async_trait]
pub trait PluginLoader: Send + Sync {
    async fn load(
        &self,
        plugin: &PluginManifest,
        settings: Option<&PluginSettings>,
    ) -> Result<PluginHandle, PluginError>;
    /// Ends the instance for good: the handle is retired ([`PluginHandle::retire`]) and every
    /// clone of it refuses further calls. A busy plugin is refused with [`PluginError::Busy`]
    /// and stays loaded.
    async fn unload(&self, instance: PluginHandle) -> Result<(), PluginError>;
    /// Names of the host functions this loader can link into a plugin: what a manifest's
    /// `api.hostFunctions` is checked against before the plugin is loaded.
    fn host_functions(&self) -> Vec<&'static str>;
}

#[async_trait]
pub trait PluginStorage: Send + Sync {
    async fn add(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError>;
    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError>;
    async fn get(&self, plugin_id: &str) -> Result<Plugin, PluginError>;
    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError>;
    /// Puts the extracted plugin in place of the installed one of its id, all at once: on an
    /// error the installed plugin is as it was. The installed one, with the core's files about
    /// it, is kept aside until [`Self::discard_replaced`] or [`Self::restore_replaced`].
    async fn replace(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError>;
    /// Brings back the plugin [`Self::replace`] kept aside, dropping its replacement.
    async fn restore_replaced(&self, plugin_id: &str) -> Result<(), PluginError>;
    /// Drops the plugin [`Self::replace`] kept aside.
    async fn discard_replaced(&self, plugin_id: &str) -> Result<(), PluginError>;
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
    async fn extract(&self, file_path: &Path) -> Result<ExtractedPlugin, PluginProviderError>;
}

#[async_trait]
pub trait PluginSyncService: Send + Sync {
    async fn execute(&self) -> Result<(), PluginError>;
}

#[async_trait]
pub trait PluginDisableService: Send + Sync {
    /// Ends the plugin and forgets it as enabled: it is not loaded on the next start.
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError>;

    /// Ends the running instance only. The plugin stays enabled, for a caller that is about to
    /// load it again, as a changed build of the plugin.
    async fn unload(&self, plugin_id: String) -> Result<(), PluginError>;
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
pub trait ReloadPluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: String) -> Result<(), PluginError>;
}

#[async_trait]
pub trait GrantPluginFolderAccessUseCasePort: Send + Sync {
    async fn execute(
        &self,
        plugin_id: String,
        file: PathBuf,
        dry_run: bool,
    ) -> Result<FolderAccess, PluginError>;
}

#[async_trait]
pub trait GrantPluginHostAccessUseCasePort: Send + Sync {
    async fn execute(
        &self,
        plugin_id: String,
        url: String,
        dry_run: bool,
    ) -> Result<HostAccess, PluginError>;
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
