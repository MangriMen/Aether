use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum InstanceErrorDto {
    Storage(String),
    NotFound {
        instance_id: String,
    },
    ContentProviderNotFound {
        plugin_id: String,
        capability_id: String,
    },
    CapabilityOperationError,
    InstanceStillInstalling {
        instance_id: String,
    },
    InstanceAlreadyRunning {
        instance_id: String,
        process_id: String,
    },
    PrelaunchCommandError {
        code: i32,
    },
    ValidationError {
        field: String,
        reason: String,
    },
    PackInfoNotFound,
    UnmanagedInstance {
        instance_id: String,
    },
    ContentDuplication {
        content_path: String,
    },
    ContentFilename {
        path: String,
    },
    ContentDownloadError(String),
    ContentProviderError {
        reason: String,
    },
    ContentForGameVersionNotFound {
        game_version: String,
    },
    UnsupportedContentType {
        content_type: String,
    },
    UnsupportedOperation(String),
    PackManagerNotFound {
        plugin_id: String,
        capability_id: String,
    },
    PackInstallFailed {
        plugin_id: String,
        capability_id: String,
    },
    PackUpdateFailed {
        modpack_id: String,
    },
}
