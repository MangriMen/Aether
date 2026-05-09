use serr::SerializeError;

use crate::features::auth::AuthDomainError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum AuthApplicationError {
    #[error(transparent)]
    Domain(#[from] AuthDomainError),

    #[error("Storage error: {0}")]
    Storage(String),
}
