use std::path::PathBuf;

use serr::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum JavaDomainError {
    #[error("No JRE found for version {version}")]
    NotFound { version: u32 },

    #[error("No JRE found at path: {path:?}")]
    InvalidPath { path: PathBuf },

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
}
