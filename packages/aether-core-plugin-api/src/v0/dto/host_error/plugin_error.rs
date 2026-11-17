use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::LoadConfigTypeDto;

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PluginErrorDto {
    NotFound {
        plugin_id: String,
    },
    AlreadyLoaded {
        plugin_id: String,
    },
    LoadingInProgress {
        plugin_id: String,
    },
    UnloadingInProgress {
        plugin_id: String,
    },
    AlreadyUnloaded {
        plugin_id: String,
    },
    LoadFailed {
        plugin_id: String,
        reason: String,
    },
    IncompatibleApiVersion {
        plugin_id: String,
        reason: String,
    },
    FunctionCallFailed {
        function_name: String,
        plugin_id: String,
        error: PluginCallErrorDto,
    },
    Manifest(ManifestErrorDto),
    ManifestNotFound {
        path: String,
    },
    InvalidManifestFormat {
        error: String,
    },
    LoaderNotFound {
        config_type: LoadConfigTypeDto,
    },
    ExtractionFailed {
        from: String,
    },
    InvalidExtractionFormat,
    FileExtractionFailed {
        from: String,
    },
    CapabilityRegistrationFailed {
        capability_type: String,
        capability_id: String,
    },
    CapabilityCancelRegistrationFailed {
        capability_type: String,
        capability_id: String,
    },
    AccessViolation {
        plugin_id: String,
        path: String,
    },
    /// The user did not allow the plugin's command, or was not there to.
    CommandNotApproved {
        plugin_id: String,
        reason: CommandRefusalDto,
    },
    HashComputationFailed,
    Storage(String),
    ProviderFetchError {
        source_type: PluginSourceTypeDto,
        details: String,
    },
    ProviderNoAssets {
        source_type: PluginSourceTypeDto,
    },
    NoRemoteSource {
        plugin_id: String,
    },
    DownloadFailed {
        url: String,
        details: String,
    },
    ProviderRateLimited {
        source_type: PluginSourceTypeDto,
        retry_after: Option<u32>,
    },
}

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PluginSourceTypeDto {
    GitHub,
    Local,
}

/// Why a command was not run.
#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CommandRefusalDto {
    /// The user said no, or said it earlier and the plugin has not been reloaded since.
    Denied,
    /// Nobody answered in time.
    TimedOut,
    /// The plugin was stopped or removed.
    Cancelled,
    /// The jar changed while the user was being asked.
    JarChanged,
    /// A value this version of the API does not know: a newer launcher sent it. Read it, never
    /// write it.
    #[serde(other)]
    Unknown,
}

/// Why a call into a plugin failed.
#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PluginCallErrorDto {
    MissingFunction,
    Timeout,
    OutOfFuel,
    OutOfMemory,
    Trap(String),
    NonZeroExit(i32),
    Plugin(String),
    /// The plugin reported that a command it asked for was not run, with the plugin's own words.
    CommandNotApproved {
        reason: CommandRefusalDto,
        details: String,
    },
    Codec(String),
    Reentrant,
}

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ManifestErrorDto {
    UnsupportedApi,
    InvalidSemver {
        field: String,
        value: String,
        error: String,
    },
    InvalidPathMapping,
    UnknownHostFunctions(Vec<String>),
    HostFunctionsNotDeclared,
    InvalidId {
        field: String,
        value: String,
    },
    ReservedPluginId,
    PlatformWithoutResolveFile {
        capability_id: String,
    },
}
