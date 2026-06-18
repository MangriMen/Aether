use serde::{Deserialize, Serialize};

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
    FunctionCallFailed {
        function_name: String,
        plugin_id: String,
        error: String,
    },
    ManifestNotFound {
        path: String,
    },
    InvalidManifestFormat {
        error: String,
    },
    LoaderNotFound {
        config_type: String,
    },
    InvalidConfig {
        config: String,
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
    Settings(String),
    Storage(String),
}
