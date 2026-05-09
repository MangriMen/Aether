use aether_core::features::java::{app::JavaApplicationError, domain::JavaDomainError};
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JavaErrorDto {
    NotFound {
        version: u32,
    },
    InvalidPath {
        path: String,
    },
    InvalidVersion {
        version: String,
    },
    FailedToGetProperties {
        reason: String,
    },
    VersionNotAvailable {
        version: u32,
        os: String,
        arch: String,
    },
    VersionGetFailed {
        version: u32,
        os: String,
        arch: String,
    },
    FailedToInstall {
        version: u32,
        os: String,
        arch: String,
    },
    RemoveOldInstallationError {
        path: String,
    },
    DownloadFailed {
        details: String,
    },
    StorageError {
        details: String,
    },
}

impl From<&JavaApplicationError> for JavaErrorDto {
    fn from(value: &JavaApplicationError) -> Self {
        match value {
            JavaApplicationError::Domain(domain) => Self::from(domain),
            JavaApplicationError::DownloadFailed(err) => Self::DownloadFailed {
                details: err.to_string(),
            },
            JavaApplicationError::Storage(err) => Self::StorageError {
                details: err.to_string(),
            },
        }
    }
}

impl From<&JavaDomainError> for JavaErrorDto {
    fn from(value: &JavaDomainError) -> Self {
        match value {
            JavaDomainError::NotFound { version } => Self::NotFound { version: *version },
            JavaDomainError::InvalidPath { path } => Self::InvalidPath {
                path: path.to_string_lossy().to_string(),
            },
            JavaDomainError::InvalidVersion { version } => Self::InvalidVersion {
                version: version.clone(),
            },
            JavaDomainError::FailedToGetProperties { reason } => Self::FailedToGetProperties {
                reason: reason.clone(),
            },
            JavaDomainError::VersionNotAvailable { version, os, arch } => {
                Self::VersionNotAvailable {
                    version: *version,
                    os: os.clone(),
                    arch: arch.clone(),
                }
            }
            JavaDomainError::VersionGetFailed { version, os, arch } => Self::VersionGetFailed {
                version: *version,
                os: os.clone(),
                arch: arch.clone(),
            },
            JavaDomainError::FailedToInstall { version, os, arch } => Self::FailedToInstall {
                version: *version,
                os: os.clone(),
                arch: arch.clone(),
            },
            JavaDomainError::RemoveOldInstallationError { path } => {
                Self::RemoveOldInstallationError {
                    path: path.to_string_lossy().to_string(),
                }
            }
        }
    }
}
