use serde::{Deserialize, Serialize};

use crate::v0::{ContentTypeDto, ProviderIdDto};

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
        field: InstanceFieldDto,
        reason: InstanceValidationErrorReasonDto,
    },
    ImporterNotFound {
        importer_id: String,
    },
    ImportFailed {
        plugin_id: String,
        capability_id: String,
        reason: String,
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
    InvalidRelativePath {
        path: String,
        reason: String,
    },
    PathEscapesInstance {
        path: String,
        instance_id: String,
    },
    FileTooLarge {
        path: String,
        size: u64,
        limit: u64,
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
    UnsupportedOperation {
        provider_id: ProviderIdDto,
        operation: String,
    },
    PlatformProviderNotFound {
        platform: String,
    },
    PlatformProviderNotChosen {
        platform: String,
    },
    GameChangeNotAllowed {
        instance_id: String,
        provider_id: ProviderIdDto,
        content_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceFieldDto {
    Name,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceValidationErrorReasonDto {
    CannotBeEmpty,
}
