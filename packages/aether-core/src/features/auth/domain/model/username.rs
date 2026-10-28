use std::fmt;

use super::super::AuthDomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    pub(crate) fn reconstitute(s: String) -> Self {
        Self(s)
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
    use rstest::rstest;

    #[rstest]
    #[case::length_is_valid("Player123")]
    #[case::contains_underscores("my_player")]
    #[case::length_is_the_minimum("abc")]
    #[case::length_is_the_maximum("aaaaaaaaaaaaaaaa")]
    fn should_parse_valid_username(#[case] input: &str) {
        let username = Username::parse(input).unwrap();
        assert_eq!(username.as_str(), input);
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

    #[rstest]
    #[case::special_chars("user@name")]
    #[case::spaces("my user")]
    #[case::cyrillic("игрок")]
    fn should_fail_validation_when_username_contains_invalid_chars(#[case] input: &str) {
        let err = Username::parse(input).unwrap_err();
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
        assert!(matches!(
            result,
            Err(AuthDomainError::InvalidUsernameLength { .. })
        ));
    }
}
