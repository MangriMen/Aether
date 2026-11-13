use aether_core::features::java::{JavaApplicationError, JavaDomainError};
use serde::Serialize;
use specta::Type;

use super::{CauseDto, FrontendError};

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JavaErrorDto {
    NotFound {
        version: u32,
    },
    InvalidPath {
        path: String,
    },
    EmptyPath,
    RelativePath {
        path: String,
    },
    InvalidVersion {
        version: String,
    },
    FailedToGetProperties {
        path: String,
        cause: CauseDto,
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
    StorageError {
        cause: CauseDto,
    },
    Cancelled,
}

impl From<&JavaApplicationError> for FrontendError {
    /// A failed download is a request error: the frontend translates it by what went wrong
    /// with the request, not by who asked.
    fn from(value: &JavaApplicationError) -> Self {
        match value {
            JavaApplicationError::Domain(domain) => Self::Java(domain.into()),
            JavaApplicationError::DownloadFailed(err) => Self::Request(err.into()),
        }
    }
}

impl From<&JavaDomainError> for JavaErrorDto {
    fn from(value: &JavaDomainError) -> Self {
        match value {
            JavaDomainError::NotFound { version } => Self::NotFound { version: *version },
            JavaDomainError::EmptyPath => Self::EmptyPath,
            JavaDomainError::RelativePath { path } => Self::RelativePath { path: path.clone() },
            JavaDomainError::InvalidPath { path } => Self::InvalidPath {
                path: path.to_string_lossy().to_string(),
            },
            JavaDomainError::InvalidVersion { version } => Self::InvalidVersion {
                version: version.clone(),
            },
            JavaDomainError::FailedToGetProperties { path, source } => {
                Self::FailedToGetProperties {
                    path: path.to_string_lossy().to_string(),
                    cause: source.into(),
                }
            }
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
            JavaDomainError::Cancelled => Self::Cancelled,
            JavaDomainError::Storage(err) => Self::StorageError { cause: err.into() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::shared::request_client::RequestError;
    use serde_json::{Value, json};
    use std::path::PathBuf;

    fn domain(err: &JavaDomainError) -> Value {
        serde_json::to_value(JavaErrorDto::from(err)).unwrap()
    }

    #[test]
    fn should_serialize_failed_properties_with_the_path_and_an_io_cause() {
        let path = PathBuf::from("jre").join("java");
        let out = domain(&JavaDomainError::FailedToGetProperties {
            path: path.clone(),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        });

        assert_eq!(
            out,
            json!({
                "code": "FAILED_TO_GET_PROPERTIES",
                "payload": {
                    "path": path.to_string_lossy(),
                    "cause": {"type": "io", "kind": "permissionDenied"}
                }
            })
        );
    }

    #[test]
    fn should_serialize_a_storage_error_with_a_database_cause() {
        let out = domain(&JavaDomainError::Storage(sqlx::Error::RowNotFound));

        assert_eq!(
            out,
            json!({"code": "STORAGE_ERROR", "payload": {"cause": {"type": "database"}}})
        );
    }

    #[test]
    fn should_keep_a_domain_java_error_as_a_java_error() {
        let err = JavaApplicationError::Domain(JavaDomainError::Cancelled);

        assert!(matches!(
            FrontendError::from(&err),
            FrontendError::Java(JavaErrorDto::Cancelled)
        ));
    }

    #[test]
    fn should_report_a_failed_download_as_a_request_error() {
        let err = JavaApplicationError::DownloadFailed(RequestError::Status {
            status: 503,
            url: "https://example.com/jre".into(),
        });

        let FrontendError::Request(dto) = FrontendError::from(&err) else {
            panic!("expected a request error");
        };

        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            json!({"code": "STATUS", "payload": {"status": 503, "url": "https://example.com/jre"}})
        );
    }
}
