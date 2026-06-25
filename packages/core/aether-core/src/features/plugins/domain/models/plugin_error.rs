use crate::{
    features::{
        plugins::{LoadConfig, LoadConfigType, ManifestError},
        settings::SettingsError,
    },
    shared::io::domain::IoError,
};

#[derive(thiserror::Error, Debug)]
pub enum PluginError {
    // Plugin lifecycle errors
    #[error("Plugin \"{plugin_id}\" not found")]
    NotFound { plugin_id: String },

    #[error("Plugin \"{plugin_id}\" is already loaded")]
    AlreadyLoaded { plugin_id: String },

    #[error("Plugin \"{plugin_id}\" is currently loading")]
    LoadingInProgress { plugin_id: String },

    #[error("Plugin \"{plugin_id}\" is currently unloading")]
    UnloadingInProgress { plugin_id: String },

    #[error("Plugin \"{plugin_id}\" is already unloaded")]
    AlreadyUnloaded { plugin_id: String },

    #[error("Failed to load plugin \"{plugin_id}\": {reason}")]
    LoadFailed { plugin_id: String, reason: String },

    // API version compatibility errors
    #[error("Plugin \"{plugin_id}\" requires an incompatible API version: {reason}")]
    IncompatibleApiVersion { plugin_id: String, reason: String },

    // Plugin execution errors
    #[error("Function \"{function_name}\" call failed in plugin \"{plugin_id}\": {error}")]
    FunctionCallFailed {
        function_name: String,
        plugin_id: String,
        error: String,
    },

    // Configuration & loading errors
    #[error("Plugin manifest not found: {path}")]
    ManifestNotFound { path: String },

    #[error("Invalid plugin manifest format: {error}")]
    InvalidManifestFormat { error: String },

    #[error("{0}")]
    Manifest(#[from] ManifestError),

    #[error("Loader not found for config type: {config_type:?}")]
    LoaderNotFound { config_type: LoadConfigType },

    #[error("Invalid load configuration: {config:?}")]
    InvalidConfig { config: LoadConfig },

    #[error("Failed to extract plugin from: {from}")]
    ExtractionFailed { from: String },

    #[error("Invalid extraction format")]
    InvalidExtractionFormat,

    #[error("Failed to extract plugin files: {from}")]
    FileExtractionFailed { from: String },

    #[error("Failed to register capability {capability_type} with id \"{capability_id}\"")]
    CapabilityRegistrationFailed {
        capability_type: &'static str,
        capability_id: String,
    },

    #[error(
        "Failed to cancel capability registration {capability_type} with id \"{capability_id}\""
    )]
    CapabilityCancelRegistrationFailed {
        capability_type: &'static str,
        capability_id: String,
    },

    // Security & access errors
    #[error("Plugin \"{plugin_id}\" attempted to access restricted path: {path}")]
    AccessViolation { plugin_id: String, path: String },

    // Infrastructure errors
    #[error("Failed to compute hash")]
    HashComputationFailed,

    #[error("Settings update failed: {0}")]
    Settings(#[from] SettingsError),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] IoError),

    // ── GitHub / Remote source errors ──
    #[error("Failed to fetch GitHub release info for {owner}/{repo}: {details}")]
    GitHubFetchError {
        owner: String,
        repo: String,
        details: String,
    },

    #[error("No release assets found for tag \"{tag}\" in {owner}/{repo}: expected a .zip file")]
    GitHubNoAssets {
        owner: String,
        repo: String,
        tag: String,
    },

    #[error("Plugin \"{plugin_id}\" has no GitHub source configured")]
    NotAGitHubPlugin { plugin_id: String },

    #[error("Failed to download plugin from {url}: {details}")]
    DownloadFailed { url: String, details: String },
}
