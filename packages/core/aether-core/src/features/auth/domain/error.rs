use serr::SerializeError;
use uuid::Uuid;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum AuthDomainError {
    #[error("Credentials with id: {id} not found")]
    CredentialsNotFound { id: Uuid },

    #[error("Active credentials not found")]
    NoActiveCredentials,

    #[error("Invalid username length, min: {min}, max: {max}")]
    InvalidUsernameLength { min: usize, max: usize },

    #[error("Invalid username chars")]
    InvalidUsernameChars,

    #[error("Session expired, please login again")]
    TokenExpired,
}
