use std::path::PathBuf;

use serr::SerializeError;

use super::LoaderVersionPreference;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum MinecraftDomainError {
    #[error("Minecraft version \"{version}\" not found")]
    VersionNotFound { version: String },

    #[error("Minecraft version not found for loader version {loader_version_preference:?}")]
    VersionForLoaderNotFound {
        loader_version_preference: LoaderVersionPreference,
    },

    #[error("Loader version {loader_version_preference:?} not found")]
    LoaderNotFound {
        loader_version_preference: LoaderVersionPreference,
    },

    #[error("Loader version not specified for non-vanilla loader")]
    LoaderVersionRequired,

    #[error("Stable or latest loader could not be resolved")]
    DefaultLoaderNotFound,

    #[error("Modloader processor failed: {reason}")]
    ProcessorFailed { reason: String },

    #[error("Failed to execute pre-launch command: exit code {code}")]
    PreLaunchFailed { code: i32 },

    #[error("Failed to execute post-launch command: exit code {code}")]
    PostLaunchFailed { code: i32 },

    #[error("Path not found: {path:?} ({entity_type})")]
    PathNotFound { path: PathBuf, entity_type: String },

    #[error("Error while parsing libraries: {reason}")]
    ParseFailed { reason: String },

    #[error("Storage failure: {reason}")]
    StorageFailure { reason: String },
}
