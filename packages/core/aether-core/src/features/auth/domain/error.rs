use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AuthDomainError {
    #[error("Credentials with id: {id} not found")]
    CredentialsNotFound { id: Uuid },

    #[error("Active credentials not found")]
    NoActiveCredentials,

    #[error("Invalid username length, min: {min}, max: {max}")]
    InvalidUsernameLength { min: usize, max: usize },

    #[error("Invalid username chars")]
    InvalidUsernameChars,

    #[error("Invalid account type")]
    InvalidAccountType,

    #[error("Session expired, please login again")]
    TokenExpired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_not_found_format() {
        let id = Uuid::nil();
        let err = AuthDomainError::CredentialsNotFound { id };
        assert_eq!(
            err.to_string(),
            format!("Credentials with id: {id} not found")
        );
    }

    #[test]
    fn no_active_credentials_format() {
        let err = AuthDomainError::NoActiveCredentials;
        assert_eq!(err.to_string(), "Active credentials not found");
    }

    #[test]
    fn invalid_username_length_format() {
        let err = AuthDomainError::InvalidUsernameLength { min: 3, max: 16 };
        assert_eq!(err.to_string(), "Invalid username length, min: 3, max: 16");
    }

    #[test]
    fn invalid_username_chars_format() {
        let err = AuthDomainError::InvalidUsernameChars;
        assert_eq!(err.to_string(), "Invalid username chars");
    }

    #[test]
    fn invalid_account_type_format() {
        let err = AuthDomainError::InvalidAccountType;
        assert_eq!(err.to_string(), "Invalid account type");
    }

    #[test]
    fn token_expired_format() {
        let err = AuthDomainError::TokenExpired;
        assert_eq!(err.to_string(), "Session expired, please login again");
    }
}
