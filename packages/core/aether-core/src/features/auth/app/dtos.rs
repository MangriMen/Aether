use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::auth::domain::{AccountType, Credential};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub id: Uuid,
    pub username: String,
    pub account_type: AccountType,
    pub active: bool,
}

impl From<&Credential> for AccountData {
    fn from(credentials: &Credential) -> Self {
        Self {
            id: credentials.id(),
            username: credentials.username().to_string(),
            account_type: credentials.account_type(),
            active: credentials.is_active(),
        }
    }
}

impl From<Credential> for AccountData {
    fn from(credentials: Credential) -> Self {
        AccountData::from(&credentials)
    }
}
