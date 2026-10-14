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
    fn should_format_credentials_not_found_error_when_displayed() {
        let id = Uuid::nil();
        let err = AuthDomainError::CredentialsNotFound { id };
        assert_eq!(
            err.to_string(),
            format!("Credentials with id: {id} not found")
        );
    }

    #[test]
    fn should_format_no_active_credentials_error_when_displayed() {
        let err = AuthDomainError::NoActiveCredentials;
        assert_eq!(err.to_string(), "Active credentials not found");
    }

    #[test]
    fn should_format_invalid_username_length_error_when_displayed() {
        let err = AuthDomainError::InvalidUsernameLength { min: 3, max: 16 };
        assert_eq!(err.to_string(), "Invalid username length, min: 3, max: 16");
    }

    #[test]
    fn should_format_invalid_username_chars_error_when_displayed() {
        let err = AuthDomainError::InvalidUsernameChars;
        assert_eq!(err.to_string(), "Invalid username chars");
    }

    #[test]
    fn should_format_invalid_account_type_error_when_displayed() {
        let err = AuthDomainError::InvalidAccountType;
        assert_eq!(err.to_string(), "Invalid account type");
    }

    #[test]
    fn should_format_token_expired_error_when_displayed() {
        let err = AuthDomainError::TokenExpired;
        assert_eq!(err.to_string(), "Session expired, please login again");
    }
}
