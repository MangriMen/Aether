use std::path::PathBuf;

use super::LoaderVersionPreference;

#[derive(Debug, thiserror::Error)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_not_found_format() {
        let err = MinecraftDomainError::VersionNotFound {
            version: "1.20".into(),
        };
        assert_eq!(err.to_string(), "Minecraft version \"1.20\" not found");
    }

    #[test]
    fn loader_version_required_format() {
        let err = MinecraftDomainError::LoaderVersionRequired;
        assert_eq!(
            err.to_string(),
            "Loader version not specified for non-vanilla loader"
        );
    }

    #[test]
    fn processor_failed_format() {
        let err = MinecraftDomainError::ProcessorFailed {
            reason: "out of memory".into(),
        };
        assert_eq!(err.to_string(), "Modloader processor failed: out of memory");
    }

    #[test]
    fn pre_launch_failed_format() {
        let err = MinecraftDomainError::PreLaunchFailed { code: -1 };
        assert_eq!(
            err.to_string(),
            "Failed to execute pre-launch command: exit code -1"
        );
    }

    #[test]
    fn storage_failure_format() {
        let err = MinecraftDomainError::StorageFailure {
            reason: "disk full".into(),
        };
        assert_eq!(err.to_string(), "Storage failure: disk full");
    }
}
