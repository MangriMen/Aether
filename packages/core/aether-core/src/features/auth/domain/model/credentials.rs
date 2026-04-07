use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::auth::AuthDomainError;

use super::Username;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    id: Uuid,
    pub username: Username,
    account_type: AccountType,
    active: bool,
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

impl Credentials {
    pub fn new_offline(id: Uuid, username: Username) -> Self {
        Self {
            id,
            username,
            active: false,
            account_type: AccountType::Offline,
            access_token: "null".to_string(),
            refresh_token: "null".to_string(),
            expires: Utc::now() + Duration::days(365 * 99),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn account_type(&self) -> AccountType {
        self.account_type
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn access_token(&self) -> &str {
        &self.access_token
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
        };

        self.active = true;

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn update_tokens(&mut self, access: String, refresh: String, expires_in: i64) {
        self.access_token = access;
        self.refresh_token = refresh;
        self.expires = Utc::now() + chrono::Duration::seconds(expires_in);
    }
}
