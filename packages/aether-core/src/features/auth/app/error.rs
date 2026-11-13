use crate::features::auth::domain::AuthDomainError;

#[derive(Clone, Debug, thiserror::Error)]
pub enum AuthApplicationError {
    #[error(transparent)]
    Domain(#[from] AuthDomainError),

    #[error("Storage error: {0}")]
    Storage(String),
}
