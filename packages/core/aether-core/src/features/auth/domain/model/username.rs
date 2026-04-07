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
        if !(MIN_USERNAME_LENGTH..=MAX_USERNAME_LENGTH).contains(&s.chars().count()) {
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
