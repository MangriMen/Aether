use std::fmt;

use serde::{Deserialize, Serialize};

use super::super::AuthDomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String")]
pub struct Username(String);

const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 16;

impl Username {
    pub fn parse(s: &str) -> Result<Self, AuthDomainError> {
        if !(MIN_USERNAME_LENGTH..=MAX_USERNAME_LENGTH).contains(&s.len()) {
            return Err(AuthDomainError::InvalidUsernameLength {
                min: MIN_USERNAME_LENGTH,
                max: MAX_USERNAME_LENGTH,
            });
        }

        if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(AuthDomainError::InvalidUsernameChars);
        }

        Ok(Self(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Username {
    type Error = AuthDomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_username_when_length_is_valid() {
        let username = Username::parse("Player123").unwrap();
        assert_eq!(username.as_str(), "Player123");
    }

    #[test]
    fn should_parse_username_with_underscores() {
        let username = Username::parse("my_player").unwrap();
        assert_eq!(username.as_str(), "my_player");
    }

    #[test]
    fn should_fail_validation_when_username_is_too_short() {
        let err = Username::parse("ab").unwrap_err();
        assert!(
            matches!(err, AuthDomainError::InvalidUsernameLength { min: 3, .. }),
            "expected InvalidUsernameLength, got {err:?}"
        );
    }

    #[test]
    fn should_fail_validation_when_username_is_too_long() {
        let long = "a".repeat(17);
        let err = Username::parse(&long).unwrap_err();
        assert!(
            matches!(err, AuthDomainError::InvalidUsernameLength { max: 16, .. }),
            "expected InvalidUsernameLength, got {err:?}"
        );
    }

    #[test]
    fn should_fail_validation_when_username_contains_special_chars() {
        let err = Username::parse("user@name").unwrap_err();
        assert!(matches!(err, AuthDomainError::InvalidUsernameChars));
    }

    #[test]
    fn should_fail_validation_when_username_contains_spaces() {
        let err = Username::parse("my user").unwrap_err();
        assert!(matches!(err, AuthDomainError::InvalidUsernameChars));
    }

    #[test]
    fn should_fail_validation_when_username_contains_cyrillic() {
        let err = Username::parse("игрок").unwrap_err();
        assert!(matches!(err, AuthDomainError::InvalidUsernameChars));
    }

    #[test]
    fn should_convert_from_string_when_valid() {
        let username: Username = "ValidUser".to_string().try_into().unwrap();
        assert_eq!(username.as_str(), "ValidUser");
    }

    #[test]
    fn should_fail_conversion_from_string_when_invalid() {
        let result: Result<Username, AuthDomainError> = "ab".to_string().try_into();
        assert!(result.is_err());
    }

    #[test]
    fn should_display_username() {
        let username = Username::parse("DisplayMe").unwrap();
        assert_eq!(format!("{username}"), "DisplayMe");
    }

    #[test]
    fn should_return_str_from_as_ref() {
        let username = Username::parse("RefTest").unwrap();
        assert_eq!(username.as_ref(), "RefTest");
    }

    #[test]
    fn should_support_minimum_length_boundary() {
        let username = Username::parse("abc").unwrap();
        assert_eq!(username.as_str(), "abc");
    }

    #[test]
    fn should_support_maximum_length_boundary() {
        let long = "a".repeat(16);
        let username = Username::parse(&long).unwrap();
        assert_eq!(username.as_str(), long);
    }
}
