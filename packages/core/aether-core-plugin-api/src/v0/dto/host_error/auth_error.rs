use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum AuthErrorDto {
    CredentialsNotFound { id: String },
    NoActiveCredentials,
    InvalidUsernameLength { min: usize, max: usize },
    InvalidUsernameChars,
    InvalidAccountType,
    TokenExpired,
    Storage(String),
}
