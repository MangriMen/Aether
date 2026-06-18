use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum JavaDomainError {
    #[error("No JRE found for version {version}")]
    NotFound { version: u32 },

    #[error("No JRE found at path: {path:?}")]
    InvalidPath { path: PathBuf },

    #[error("Path can't be empty for edit operations")]
    EmptyPath,

    #[error("Invalid JRE version: {version}")]
    InvalidVersion { version: String },

    #[error("Failed to get java properties: {reason}")]
    FailedToGetProperties { reason: String },

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
    },

    #[error("Failed to remove old installation at {path:?}")]
    RemoveOldInstallationError { path: PathBuf },

    #[error("Storage error: {0}")]
    Storage(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_format() {
        let err = JavaDomainError::NotFound { version: 17 };
        assert_eq!(err.to_string(), "No JRE found for version 17");
    }

    #[test]
    fn invalid_version_format() {
        let err = JavaDomainError::InvalidVersion {
            version: "abc".into(),
        };
        assert_eq!(err.to_string(), "Invalid JRE version: abc");
    }

    #[test]
    fn storage_format() {
        let err = JavaDomainError::Storage("db error".into());
        assert_eq!(err.to_string(), "Storage error: db error");
    }

    #[test]
    fn empty_path_format() {
        let err = JavaDomainError::EmptyPath;
        assert_eq!(err.to_string(), "Path can't be empty for edit operations");
    }
}
