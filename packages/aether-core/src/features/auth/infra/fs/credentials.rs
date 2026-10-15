use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::auth::{AccountType, Credential};

use super::UsernameV1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialV1 {
    pub id: Uuid,
    pub username: UsernameV1,
    pub account_type: AccountTypeV1,
    #[serde(alias = "active")]
    pub is_active: bool,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountTypeV1 {
    Offline,
    Microsoft,
}

impl From<AccountTypeV1> for AccountType {
    fn from(value: AccountTypeV1) -> Self {
        match value {
            AccountTypeV1::Offline => Self::Offline,
            AccountTypeV1::Microsoft => Self::Microsoft,
        }
    }
}

impl From<AccountType> for AccountTypeV1 {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Offline => Self::Offline,
            AccountType::Microsoft => Self::Microsoft,
        }
    }
}

impl From<Credential> for CredentialV1 {
    fn from(value: Credential) -> Self {
        Self {
            id: value.id(),
            username: value.username().into(),
            account_type: value.account_type().into(),
            is_active: value.is_active(),
            access_token: value.access_token().to_owned(),
            refresh_token: value.refresh_token().to_owned(),
            expires: value.expires(),
        }
    }
}

impl From<CredentialV1> for Credential {
    fn from(value: CredentialV1) -> Self {
        Self::new(
            value.id,
            value.username.into(),
            value.account_type.into(),
            value.is_active,
            value.access_token,
            value.refresh_token,
            value.expires,
        )
    }
}
