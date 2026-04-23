use aether_core::features::auth::{AuthApplicationError, AuthDomainError};
use serde::Serialize;
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthErrorDto {
    CredentialsNotFound { id: Uuid },

    NoActiveCredentials,

    InvalidUsernameLength { min: usize, max: usize },

    InvalidUsernameChars,

    TokenExpired,

    StorageError,
}

impl From<&AuthApplicationError> for AuthErrorDto {
    fn from(value: &AuthApplicationError) -> Self {
        match value {
            AuthApplicationError::Domain(domain) => Self::from(domain),
            AuthApplicationError::StorageFailure { .. } => Self::StorageError,
        }
    }
}

impl From<&AuthDomainError> for AuthErrorDto {
    fn from(value: &AuthDomainError) -> Self {
        match value {
            AuthDomainError::CredentialsNotFound { id } => Self::CredentialsNotFound { id: *id },
            AuthDomainError::NoActiveCredentials => Self::NoActiveCredentials,
            AuthDomainError::InvalidUsernameLength { min, max } => Self::InvalidUsernameLength {
                min: *min,
                max: *max,
            },
            AuthDomainError::InvalidUsernameChars => Self::InvalidUsernameChars,
            AuthDomainError::TokenExpired => Self::TokenExpired,
        }
    }
}
