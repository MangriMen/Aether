use serr::SerializeError;

use crate::{features::auth::AuthDomainError, shared::IoError};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum AuthApplicationError {
    #[error(transparent)]
    Domain(#[from] AuthDomainError),

    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),
}
