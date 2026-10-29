use crate::{
    features::{
        instance::{CapabilityId, PluginId},
        plugins::{
            AllowedHostRejection, AllowedPathRejection, LoadConfigType, ManifestError,
            PluginCallError, PluginProviderError,
        },
        settings::SettingsError,
    },
    shared::io::domain::IoError,
};

#[derive(thiserror::Error, Debug)]
pub enum PluginError {
    // Plugin lifecycle errors
    /// The id as it was asked for: one outside the id scheme names no plugin either.
    #[error("Plugin \"{plugin_id}\" not found")]
    NotFound { plugin_id: String },

    #[error("Plugin \"{plugin_id}\" is already loaded")]
    AlreadyLoaded { plugin_id: PluginId },

    #[error("Plugin \"{plugin_id}\" is currently loading")]
    LoadingInProgress { plugin_id: PluginId },

    #[error("Plugin \"{plugin_id}\" is currently unloading")]
    UnloadingInProgress { plugin_id: PluginId },

    #[error("Plugin \"{plugin_id}\" is already unloaded")]
    AlreadyUnloaded { plugin_id: PluginId },

    #[error(
        "Plugin \"{plugin_id}\" is busy: an import, install or update is in progress. Try again once it finishes"
    )]
    Busy { plugin_id: PluginId },

    /// A call through a handle that was retired: the plugin was unloaded, or is restarting or
    /// has restarted with a new instance.
    #[error(
        "Plugin \"{plugin_id}\" is not running: it was unloaded or is restarting. Try again once it is loaded"
    )]
    Stopped { plugin_id: PluginId },

    #[error(
        "Plugin \"{plugin_id}\" cannot be updated: the new archive is of plugin \"{archive_id}\""
    )]
    IdMismatch {
        plugin_id: PluginId,
        archive_id: PluginId,
    },

    #[error("Failed to load plugin \"{plugin_id}\": {reason}")]
    LoadFailed { plugin_id: PluginId, reason: String },

    // API version compatibility errors
    #[error("Plugin \"{plugin_id}\" requires an incompatible API version: {reason}")]
    IncompatibleApiVersion { plugin_id: PluginId, reason: String },

    // Plugin execution errors
    #[error("Function \"{function_name}\" call failed in plugin \"{plugin_id}\": {error}")]
    FunctionCallFailed {
        function_name: String,
        plugin_id: PluginId,
        #[source]
        error: PluginCallError,
    },

    // Configuration & loading errors
    #[error("{0}")]
    Manifest(#[from] ManifestError),

    #[error("Loader not found for config type: {config_type:?}")]
    LoaderNotFound { config_type: LoadConfigType },

    #[error("Failed to register capability {capability_type} with id \"{capability_id}\"")]
    CapabilityRegistrationFailed {
        capability_type: &'static str,
        capability_id: CapabilityId,
    },

    #[error(
        "Failed to cancel capability registration {capability_type} with id \"{capability_id}\""
    )]
    CapabilityCancelRegistrationFailed {
        capability_type: &'static str,
        capability_id: CapabilityId,
    },

    // Security & access errors
    #[error("Plugin \"{plugin_id}\" attempted to access restricted path: {path}")]
    AccessViolation { plugin_id: PluginId, path: String },

    #[error("Folder \"{path}\" cannot be given to plugin \"{plugin_id}\": {reason}")]
    InvalidAllowedPath {
        plugin_id: PluginId,
        path: String,
        reason: AllowedPathRejection,
    },

    #[error("The host of \"{url}\" cannot be given to plugin \"{plugin_id}\": {reason}")]
    InvalidAllowedHost {
        plugin_id: PluginId,
        url: String,
        reason: AllowedHostRejection,
    },

    // Infrastructure errors
    #[error("Failed to compute hash")]
    HashComputationFailed,

    #[error("Settings update failed: {0}")]
    Settings(#[from] SettingsError),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] IoError),

    #[error(transparent)]
    Provider(#[from] PluginProviderError),
}
