use std::path::PathBuf;

use crate::shared::error::DatabaseError;

#[derive(Clone, Debug, thiserror::Error)]
pub enum JavaDomainError {
    #[error("No JRE found for version {version}")]
    NotFound { version: u32 },

    #[error("No JRE found at path: {path:?}")]
    InvalidPath { path: PathBuf },

    #[error("Path can't be empty for edit operations")]
    EmptyPath,

    #[error("Java path must be absolute: {path:?}")]
    RelativePath { path: String },

    #[error("Invalid JRE version: {version}")]
    InvalidVersion { version: String },

    #[error("The installation of the JRE was called off")]
    Cancelled,

    #[error("Failed to get java properties of {path:?}: {source}")]
    FailedToGetProperties {
        path: PathBuf,
        #[source]
        source: std::sync::Arc<std::io::Error>,
    },

    #[error("No JRE version found to download: version {version}, os {os}, architecture {arch}")]
    VersionNotAvailable {
        version: u32,
        os: String,
        arch: String,
    },

    #[error("Failed to get JRE: version {version}, os {os}, architecture {arch}")]
    VersionGetFailed {
        version: u32,
        os: String,
        arch: String,
    },

    #[error("Failed to install JRE: version {version}, os {os}, architecture {arch}")]
    FailedToInstall {
        version: u32,
        os: String,
        arch: String,
        #[source]
        source: std::sync::Arc<std::io::Error>,
    },

    #[error("Failed to remove old installation at {path:?}")]
    RemoveOldInstallationError {
        path: PathBuf,
        #[source]
        source: std::sync::Arc<std::io::Error>,
    },

    #[error("Storage error: {0}")]
    Storage(#[from] DatabaseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_not_found_error_when_displayed() {
        let err = JavaDomainError::NotFound { version: 17 };
        assert_eq!(err.to_string(), "No JRE found for version 17");
    }

    #[test]
    fn should_format_invalid_version_error_when_displayed() {
        let err = JavaDomainError::InvalidVersion {
            version: "abc".into(),
        };
        assert_eq!(err.to_string(), "Invalid JRE version: abc");
    }

    #[test]
    fn should_format_storage_error_when_displayed() {
        let err = JavaDomainError::Storage(DatabaseError::new("locked"));
        assert!(err.to_string().starts_with("Storage error: "));
    }

    #[test]
    fn should_format_empty_path_error_when_displayed() {
        let err = JavaDomainError::EmptyPath;
        assert_eq!(err.to_string(), "Path can't be empty for edit operations");
    }
}
