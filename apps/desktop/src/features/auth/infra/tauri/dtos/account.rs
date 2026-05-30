use aether_core::features::auth::{AccountData, AccountType};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub id: Uuid,
    pub username: String,
    pub account_type: AccountTypeDto,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountTypeDto {
    Offline,
    Microsoft,
}

impl From<AccountData> for AccountDto {
    fn from(core: AccountData) -> Self {
        Self {
            id: core.id,
            username: core.username,
            account_type: core.account_type.into(),
            active: core.active,
        }
    }
}

impl From<AccountType> for AccountTypeDto {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Offline => Self::Offline,
            AccountType::Microsoft => Self::Microsoft,
        }
    }
}
