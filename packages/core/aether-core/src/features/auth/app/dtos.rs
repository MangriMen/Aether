use uuid::Uuid;

use crate::features::auth::domain::{AccountType, Credential};

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub account_type: AccountType,
    pub active: bool,
}

impl From<&Credential> for Account {
    fn from(credentials: &Credential) -> Self {
        Self {
            id: credentials.id(),
            username: credentials.username().to_string(),
            account_type: credentials.account_type(),
            active: credentials.is_active(),
        }
    }
}

impl From<Credential> for Account {
    fn from(credentials: Credential) -> Self {
        Account::from(&credentials)
    }
}
