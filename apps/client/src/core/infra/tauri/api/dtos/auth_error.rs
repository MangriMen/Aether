use aether_core::features::auth::{app::AuthApplicationError, domain::AuthDomainError};
use serde::Serialize;
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthErrorDto {
    CredentialsNotFound { id: Uuid },

    NoActiveCredentials,

    InvalidUsernameLength { min: u32, max: u32 },

    InvalidUsernameChars,

    InvalidAccountType,

    TokenExpired,

    StorageError,
}

impl From<&AuthApplicationError> for AuthErrorDto {
    fn from(value: &AuthApplicationError) -> Self {
        match value {
            AuthApplicationError::Domain(domain) => Self::from(domain),
            AuthApplicationError::Storage { .. } => Self::StorageError,
        }
    }
}

impl From<&AuthDomainError> for AuthErrorDto {
    fn from(value: &AuthDomainError) -> Self {
        match value {
            AuthDomainError::CredentialsNotFound { id } => Self::CredentialsNotFound { id: *id },
            AuthDomainError::NoActiveCredentials => Self::NoActiveCredentials,
            AuthDomainError::InvalidUsernameLength { min, max } => Self::InvalidUsernameLength {
                min: (*min).try_into().unwrap_or(u32::MIN),
                max: (*max).try_into().unwrap_or(u32::MAX),
            },
            AuthDomainError::InvalidUsernameChars => Self::InvalidUsernameChars,
            AuthDomainError::TokenExpired => Self::TokenExpired,
            AuthDomainError::InvalidAccountType => Self::InvalidAccountType,
        }
    }
}
