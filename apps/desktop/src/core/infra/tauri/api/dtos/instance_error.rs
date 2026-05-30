use aether_core::features::instance::{
    InstanceError, InstanceField, InstanceValidationErrorReason,
};
use serde::Serialize;
use specta::Type;
use uuid::Uuid;

use crate::features::instance::infra::ContentTypeDto;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstanceErrorDto {
    Storage {
        details: String,
    },
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
        process_id: Uuid,
    },
    PrelaunchCommandError {
        code: i32,
    },
    ValidationError {
        field: InstanceFieldDto,
        reason: InstanceValidationErrorReasonDto,
    },
    ImporterNotFound {
        importer_id: String,
    },
    ImportFailed {
        plugin_id: String,
        capability_id: String,
    },
    PackInfoNotFound,
    UpdaterNotFound {
        modpack_id: String,
    },
    UpdateFailed {
        modpack_id: String,
    },
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
        content_type: ContentTypeDto,
    },
    SettingsLoadError {
        details: String,
    },
    MinecraftError {
        details: String,
    },
    ProcessError {
        details: String,
    },
    CredentialsError {
        details: String,
    },
}

impl From<&InstanceError> for InstanceErrorDto {
    fn from(value: &InstanceError) -> Self {
        match value {
            InstanceError::Storage(err) => Self::Storage {
                details: err.clone(),
            },
            InstanceError::NotFound { instance_id } => Self::NotFound {
                instance_id: instance_id.clone(),
            },
            InstanceError::ContentProviderNotFound {
                plugin_id,
                capability_id,
            } => Self::ContentProviderNotFound {
                plugin_id: plugin_id.clone(),
                capability_id: capability_id.clone(),
            },
            InstanceError::CapabilityOperationError => Self::CapabilityOperationError,
            InstanceError::InstanceStillInstalling { instance_id } => {
                Self::InstanceStillInstalling {
                    instance_id: instance_id.clone(),
                }
            }
            InstanceError::InstanceAlreadyRunning {
                instance_id,
                process_id,
            } => Self::InstanceAlreadyRunning {
                instance_id: instance_id.clone(),
                process_id: *process_id,
            },
            InstanceError::PrelaunchCommandError { code } => {
                Self::PrelaunchCommandError { code: *code }
            }
            InstanceError::ValidationError { field, reason } => Self::ValidationError {
                field: (*field).into(),
                reason: reason.clone().into(),
            },
            InstanceError::ImporterNotFound { importer_id } => Self::ImporterNotFound {
                importer_id: importer_id.clone(),
            },
            InstanceError::ImportFailed {
                plugin_id,
                capability_id,
            } => Self::ImportFailed {
                plugin_id: plugin_id.clone(),
                capability_id: capability_id.clone(),
            },
            InstanceError::PackInfoNotFound => Self::PackInfoNotFound,
            InstanceError::UpdaterNotFound { modpack_id } => Self::UpdaterNotFound {
                modpack_id: modpack_id.clone(),
            },
            InstanceError::UpdateFailed { modpack_id } => Self::UpdateFailed {
                modpack_id: modpack_id.clone(),
            },
            InstanceError::UnmanagedInstance { instance_id } => Self::UnmanagedInstance {
                instance_id: instance_id.clone(),
            },
            InstanceError::ContentDuplication { content_path } => Self::ContentDuplication {
                content_path: content_path.clone(),
            },
            InstanceError::ContentFilename { path } => Self::ContentFilename {
                path: path.to_string_lossy().to_string(),
            },
            InstanceError::ContentDownloadError(err) => Self::ContentDownloadError(err.clone()),
            InstanceError::ContentProviderError { reason } => Self::ContentProviderError {
                reason: reason.clone(),
            },
            InstanceError::ContentForGameVersionNotFound { game_version } => {
                Self::ContentForGameVersionNotFound {
                    game_version: game_version.clone(),
                }
            }
            InstanceError::UnsupportedContentType { content_type } => {
                Self::UnsupportedContentType {
                    content_type: (*content_type).into(),
                }
            }
            InstanceError::SettingsLoadError(err) => Self::SettingsLoadError {
                details: err.to_string(),
            },
            InstanceError::MinecraftError(err) => Self::MinecraftError {
                details: err.to_string(),
            },
            InstanceError::ProcessError(err) => Self::ProcessError {
                details: err.to_string(),
            },
            InstanceError::CredentialsError(err) => Self::CredentialsError {
                details: err.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum InstanceFieldDto {
    Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum InstanceValidationErrorReasonDto {
    CannotBeEmpty,
}

impl From<InstanceField> for InstanceFieldDto {
    fn from(value: InstanceField) -> Self {
        match value {
            InstanceField::Name => Self::Name,
        }
    }
}

impl From<InstanceValidationErrorReason> for InstanceValidationErrorReasonDto {
    fn from(value: InstanceValidationErrorReason) -> Self {
        match value {
            InstanceValidationErrorReason::CannotBeEmpty => Self::CannotBeEmpty,
        }
    }
}
