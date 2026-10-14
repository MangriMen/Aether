use serde::{Deserialize, Serialize};

use crate::v0::LoadConfigTypeDto;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginSourceTypeDto {
    GitHub,
    Local,
}

/// Why a call into a plugin failed.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum PluginCallErrorDto {
    MissingFunction,
    Timeout,
    OutOfFuel,
    OutOfMemory,
    Trap(String),
    NonZeroExit(i32),
    Plugin(String),
    Codec(String),
    Reentrant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
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
