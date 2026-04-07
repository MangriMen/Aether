use serr::SerializeError;

use crate::{
    features::java::{JavaDomainError, JavaStorageError},
    libs::request_client::RequestError,
};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum JavaApplicationError {
    #[error(transparent)]
    Domain(#[from] JavaDomainError),

    #[error("Download failed: {0}")]
    DownloadFailed(#[from] RequestError),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] JavaStorageError),
}
