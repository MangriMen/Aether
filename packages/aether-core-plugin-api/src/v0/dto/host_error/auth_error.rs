use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AuthErrorDto {
    CredentialsNotFound { id: String },
    NoActiveCredentials,
    InvalidUsernameLength { min: usize, max: usize },
    InvalidUsernameChars,
    InvalidAccountType,
    TokenExpired,
    Storage(String),
}
