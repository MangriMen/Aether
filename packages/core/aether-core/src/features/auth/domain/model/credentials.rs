use std::fmt::Display;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::auth::domain::AuthDomainError;

use super::Username;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    id: Uuid,
    username: Username,
    account_type: AccountType,
    #[serde(alias = "active")]
    is_active: bool,
    access_token: String,
    refresh_token: String,
    expires: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Offline,
    Microsoft,
}

impl Credential {
    pub fn new(
        id: Uuid,
        username: Username,
        account_type: AccountType,
        is_active: bool,
        access_token: String,
        refresh_token: String,
        expires: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            account_type,
            is_active,
            access_token,
            refresh_token,
            expires,
        }
    }

    pub fn new_offline(id: Uuid, username: Username) -> Self {
        Self {
            id,
            username,
            is_active: false,
            account_type: AccountType::Offline,
            access_token: "null".to_string(),
            refresh_token: "null".to_string(),
            expires: Utc::now() + Duration::days(365 * 99),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn username(&self) -> &Username {
        &self.username
    }
    pub fn account_type(&self) -> AccountType {
        self.account_type
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
    pub fn expires(&self) -> DateTime<Utc> {
        self.expires
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires
    }

    pub fn activate(&mut self) -> Result<(), AuthDomainError> {
        match self.account_type {
            AccountType::Microsoft => {
                if self.is_expired() {
                    return Err(AuthDomainError::TokenExpired);
                }
            }
            AccountType::Offline => (),
        }

        self.is_active = true;

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn update_tokens(&mut self, access: String, refresh: String, expires_in: i64) {
        self.access_token = access;
        self.refresh_token = refresh;
        self.expires = Utc::now() + chrono::Duration::seconds(expires_in);
    }
}

impl AccountType {
    pub fn from_string(account_type: &str) -> Result<AccountType, AuthDomainError> {
        match account_type {
            "offline" => Ok(AccountType::Offline),
            "microsoft" => Ok(AccountType::Microsoft),
            _ => Err(AuthDomainError::InvalidAccountType),
        }
    }
}

impl Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            AccountType::Offline => "offline".to_string(),
            AccountType::Microsoft => "microsoft".to_string(),
        };

        write!(f, "{string}")
    }
}

impl From<AccountType> for String {
    fn from(value: AccountType) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for AccountType {
    type Error = AuthDomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_string(&value)
    }
}
