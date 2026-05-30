use std::path::PathBuf;

use serde::Serialize;
use serr::SerializeError;
use uuid::Uuid;

use crate::features::{
    auth::AuthApplicationError, instance::ContentType, minecraft::MinecraftApplicationError,
    process::ProcessError, settings::SettingsError,
};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum InstanceError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Instance with id \"{instance_id}\" not found")]
    NotFound { instance_id: String },

    #[error(
        "Content provider not found for plugin \"{plugin_id}\" and capability \"{capability_id}\""
    )]
    ContentProviderNotFound {
        plugin_id: String,
        capability_id: String,
    },

    #[error("Capability operation error")]
    CapabilityOperationError,

    #[error("Instance \"{instance_id}\" still in installing state")]
    InstanceStillInstalling { instance_id: String },

    #[error("Instance \"{instance_id}\" already running with pid \"{process_id}\"")]
    InstanceAlreadyRunning {
        instance_id: String,
        process_id: Uuid,
    },

    #[error("Prelaunch command error with code: {code}")]
    PrelaunchCommandError { code: i32 },

    #[error("Instance validation error: \"{field:?}\" is invalid because \"{reason:?}\"")]
    ValidationError {
        field: InstanceField,
        reason: InstanceValidationErrorReason,
    },

    #[error("Not found importer {importer_id}")]
    ImporterNotFound { importer_id: String },

    #[error("Failed to import instance with {plugin_id} {capability_id}")]
    ImportFailed {
        plugin_id: String,
        capability_id: String,
    },

    // Update errors
    #[error("Not found pack info in instance")]
    PackInfoNotFound,

    #[error("Not found updater for modpack {modpack_id}")]
    UpdaterNotFound { modpack_id: String },

    #[error("Failed to update instance with modpack {modpack_id}")]
    UpdateFailed { modpack_id: String },

    #[error("Unmanaged instance")]
    UnmanagedInstance { instance_id: String },

    // Content
    #[error("Found duplicate content at {content_path}")]
    ContentDuplication { content_path: String },

    #[error("Can't get content file_name at path: {path}")]
    ContentFilename { path: PathBuf },

    #[error("Error when downloading content {0}")]
    ContentDownloadError(String),

    #[error("Content provider error {reason}")]
    ContentProviderError { reason: String },

    #[error("Not found content for minecraft version \"{game_version}\"")]
    ContentForGameVersionNotFound { game_version: String },

    #[error("Unsupported content type: {content_type:?}")]
    UnsupportedContentType { content_type: ContentType },

    // Features errors
    #[error("Settings load error")]
    #[serialize_error]
    SettingsLoadError(#[from] SettingsError),

    #[error("Failed to get launch command")]
    #[serialize_error]
    MinecraftError(#[from] MinecraftApplicationError),

    #[error("Failed to launch instance")]
    #[serialize_error]
    ProcessError(#[from] ProcessError),

    #[error(transparent)]
    #[serialize_error]
    CredentialsError(#[from] AuthApplicationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceField {
    Name,
}

// impl InstanceField {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             InstanceField::Name => "name",
//         }
//     }
// }

// impl Display for InstanceField {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.as_str())
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceValidationErrorReason {
    CannotBeEmpty,
}

// impl ValidationErrorReason {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             ValidationErrorReason::CannotBeEmpty => "cannot_be_empty",
//         }
//     }
// }

// impl Display for ValidationErrorReason {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.as_str())
//     }
// }
