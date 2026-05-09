use aether_core::features::minecraft::{
    app::MinecraftApplicationError, domain::MinecraftDomainError,
};
use serde::Serialize;
use specta::Type;

use crate::features::minecraft::LoaderVersionPreferenceDto;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MinecraftErrorDto {
    VersionNotFound {
        version: String,
    },
    VersionForLoaderNotFound {
        loader_version_preference: LoaderVersionPreferenceDto,
    },
    LoaderNotFound {
        loader_version_preference: LoaderVersionPreferenceDto,
    },
    LoaderVersionRequired,
    DefaultLoaderNotFound,
    ProcessorFailed {
        reason: String,
    },
    PreLaunchFailed {
        code: i32,
    },
    PostLaunchFailed {
        code: i32,
    },
    PathNotFound {
        path: String,
        entity_type: String,
    },
    ParseFailed {
        reason: String,
    },
    StorageFailure {
        reason: String,
    },
    JavaError {
        details: String,
    },
    Storage {
        details: String,
    },
}

impl From<&MinecraftApplicationError> for MinecraftErrorDto {
    fn from(value: &MinecraftApplicationError) -> Self {
        match value {
            MinecraftApplicationError::Domain(domain) => Self::from(domain),
            MinecraftApplicationError::JavaError(err) => Self::JavaError {
                details: err.to_string(),
            },
            MinecraftApplicationError::Storage(err) => Self::Storage {
                details: err.to_string(),
            },
        }
    }
}

impl From<&MinecraftDomainError> for MinecraftErrorDto {
    fn from(value: &MinecraftDomainError) -> Self {
        match value {
            MinecraftDomainError::VersionNotFound { version } => Self::VersionNotFound {
                version: version.clone(),
            },
            MinecraftDomainError::VersionForLoaderNotFound {
                loader_version_preference,
            } => Self::VersionForLoaderNotFound {
                loader_version_preference: loader_version_preference.clone().into(),
            },
            MinecraftDomainError::LoaderNotFound {
                loader_version_preference,
            } => Self::LoaderNotFound {
                loader_version_preference: loader_version_preference.clone().into(),
            },
            MinecraftDomainError::LoaderVersionRequired => Self::LoaderVersionRequired,
            MinecraftDomainError::DefaultLoaderNotFound => Self::DefaultLoaderNotFound,
            MinecraftDomainError::ProcessorFailed { reason } => Self::ProcessorFailed {
                reason: reason.clone(),
            },
            MinecraftDomainError::PreLaunchFailed { code } => Self::PreLaunchFailed { code: *code },
            MinecraftDomainError::PostLaunchFailed { code } => {
                Self::PostLaunchFailed { code: *code }
            }
            MinecraftDomainError::PathNotFound { path, entity_type } => Self::PathNotFound {
                path: path.to_string_lossy().to_string(),
                entity_type: entity_type.clone(),
            },
            MinecraftDomainError::ParseFailed { reason } => Self::ParseFailed {
                reason: reason.clone(),
            },
            MinecraftDomainError::StorageFailure { reason } => Self::StorageFailure {
                reason: reason.clone(),
            },
        }
    }
}
