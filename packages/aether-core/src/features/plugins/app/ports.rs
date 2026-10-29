use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use semver;

use crate::features::instance::PluginId;
use crate::features::plugins::app::{EditPluginSettings, FolderAccess, HostAccess, PluginDto};
use crate::features::plugins::domain::PluginHandle;
use crate::features::plugins::{
    ExtractedPlugin, Plugin, PluginError, PluginManifest, PluginProviderError, PluginSettings,
    PluginSource, PluginSourceType, PluginVerification, ProviderUpdateInfo, SkippedPluginDir,
};

#[async_trait]
pub trait PluginSourceStorage: Send + Sync {
    async fn save(&self, plugin_id: &PluginId, source: &PluginSource) -> Result<(), PluginError>;
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginSource>, PluginError>;
    /// No record is not an error.
    async fn remove(&self, plugin_id: &PluginId) -> Result<(), PluginError>;
}

/// Persists the trust-on-first-use record established when a plugin is fetched
/// from a remote provider. Absence of a record means "never verified", which is
/// a valid state (locally installed plugins), not an error.
#[async_trait]
pub trait PluginVerificationStorage: Send + Sync {
    async fn save(
        &self,
        plugin_id: &PluginId,
        verification: &PluginVerification,
    ) -> Result<(), PluginError>;
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginVerification>, PluginError>;
    /// Forget the record: the bytes on disk are no longer the ones it vouched for. No record
    /// is not an error.
    async fn remove(&self, plugin_id: &PluginId) -> Result<(), PluginError>;
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
    async fn list(&self) -> Result<HashMap<PluginId, Plugin>, PluginError>;
    /// Directories in the plugins folder that [`Self::list`] leaves out, with the reason, so
    /// the user can be told why such a plugin is not there.
    async fn list_skipped_dirs(&self) -> Result<Vec<SkippedPluginDir>, PluginError> {
        Ok(Vec::new())
    }
    async fn get(&self, plugin_id: &PluginId) -> Result<Plugin, PluginError>;
    async fn remove(&self, plugin_id: &PluginId) -> Result<(), PluginError>;
    /// Puts the extracted plugin in place of the installed one of its id, all at once: on an
    /// error the installed plugin is as it was. The installed one, with the core's files about
    /// it, is kept aside until [`Self::discard_replaced`] or [`Self::restore_replaced`].
    async fn replace(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError>;
    /// Brings back the plugin [`Self::replace`] kept aside, dropping its replacement.
    async fn restore_replaced(&self, plugin_id: &PluginId) -> Result<(), PluginError>;
    /// Drops the plugin [`Self::replace`] kept aside.
    async fn discard_replaced(&self, plugin_id: &PluginId) -> Result<(), PluginError>;
    /// Lowercase hex sha256 of the plugin's wasm blob as it currently sits on disk.
    async fn wasm_sha256(&self, plugin_id: &PluginId) -> Result<String, PluginError>;
}

#[async_trait]
pub trait PluginSettingsStorage: Send + Sync {
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginSettings>, PluginError>;
    async fn upsert(
        &self,
        plugin_id: &PluginId,
        settings: &PluginSettings,
    ) -> Result<(), PluginError>;
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
    async fn execute(&self, plugin_id: PluginId) -> Result<(), PluginError>;

    /// Ends the running instance only. The plugin stays enabled, for a caller that is about to
    /// load it again, as a changed build of the plugin.
    async fn unload(&self, plugin_id: PluginId) -> Result<(), PluginError>;
}

// ── Use case ports ──

#[async_trait]
pub trait CheckForPluginUpdatesUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: &PluginId) -> Result<ProviderUpdateInfo, PluginError>;
}

#[async_trait]
pub trait EditPluginSettingsUseCasePort: Send + Sync {
    async fn execute(
        &self,
        plugin_id: PluginId,
        edit_settings: EditPluginSettings,
    ) -> Result<(), PluginError>;
}

#[async_trait]
pub trait EnablePluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: PluginId) -> Result<(), PluginError>;
}

#[async_trait]
pub trait ReloadPluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: PluginId) -> Result<(), PluginError>;
}

#[async_trait]
pub trait GrantPluginFolderAccessUseCasePort: Send + Sync {
    async fn execute(
        &self,
        plugin_id: PluginId,
        file: PathBuf,
        dry_run: bool,
    ) -> Result<FolderAccess, PluginError>;
}

#[async_trait]
pub trait GrantPluginHostAccessUseCasePort: Send + Sync {
    async fn execute(
        &self,
        plugin_id: PluginId,
        url: String,
        dry_run: bool,
    ) -> Result<HostAccess, PluginError>;
}

#[async_trait]
pub trait ForceEnablePluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: PluginId) -> Result<(), PluginError>;
}

#[async_trait]
pub trait GetPluginApiVersionUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<semver::Version, PluginError>;
}

#[async_trait]
pub trait GetPluginDtoUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: PluginId) -> Result<PluginDto, PluginError>;
}

#[async_trait]
pub trait GetPluginSettingsUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: PluginId) -> Result<Option<PluginSettings>, PluginError>;
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
pub trait ListSkippedPluginDirsUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<SkippedPluginDir>, PluginError>;
}

#[async_trait]
pub trait RemovePluginUseCasePort: Send + Sync {
    async fn execute(&self, plugin_id: PluginId) -> Result<(), PluginError>;
}

#[async_trait]
pub trait UpdatePluginUseCasePort: Send + Sync {
    async fn execute(
        &self,
        plugin_id: &PluginId,
        target_tag: Option<&str>,
    ) -> Result<(), PluginError>;
}

/// The release a user picked from a provider's preview.
pub struct ProviderRelease {
    pub download_url: String,
    pub tag_name: String,
    pub version: String,
}

#[async_trait]
pub trait InstallPluginFromProviderUseCasePort: Send + Sync {
    /// Returns the id of the plugin the archive holds.
    async fn execute(
        &self,
        source_type: PluginSourceType,
        identifier: &str,
        release: ProviderRelease,
    ) -> Result<PluginId, PluginError>;
}
