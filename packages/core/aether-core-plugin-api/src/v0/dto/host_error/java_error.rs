use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum JavaErrorDto {
    NotFound {
        version: u32,
    },
    InvalidPath {
        path: String,
    },
    EmptyPath,
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
    Storage(String),
    DownloadFailed(String),
}
