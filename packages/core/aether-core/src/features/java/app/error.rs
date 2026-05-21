use serr::SerializeError;

use crate::{features::java::JavaDomainError, libs::request_client::RequestError};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum JavaApplicationError {
    #[error(transparent)]
    Domain(#[from] JavaDomainError),

    #[error("Download failed: {0}")]
    DownloadFailed(#[from] RequestError),
}
