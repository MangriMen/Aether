use serde::{Deserialize, Serialize};

use crate::features::auth::Username;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UsernameV1(pub String);

impl From<UsernameV1> for Username {
    fn from(value: UsernameV1) -> Self {
        Self::reconstitute(value.0)
    }
}

impl From<Username> for UsernameV1 {
    fn from(value: Username) -> Self {
        Self(value.to_string())
    }
}

impl From<&Username> for UsernameV1 {
    fn from(value: &Username) -> Self {
        Self(value.to_string())
    }
}
