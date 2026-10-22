use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::features::auth::domain::{AccountType, Credential, Username};

/// Builder for constructing test Credentials with customizable fields.
/// Defaults: random UUID, inactive status, Offline account type.
pub struct CredentialsBuilder {
    id: Uuid,
    username: String,
    active: bool,
    account_type: AccountType,
    expires: Option<DateTime<Utc>>,
}

impl CredentialsBuilder {
    pub fn new(username: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            active: false,
            account_type: AccountType::Offline,
            expires: None,
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn with_microsoft(mut self, expires: DateTime<Utc>) -> Self {
        self.account_type = AccountType::Microsoft;
        self.expires = Some(expires);
        self
    }

    pub fn build(self) -> Credential {
        let username = Username::parse(&self.username).expect("Invalid username in test builder");

        let mut credentials = match self.account_type {
            AccountType::Offline => Credential::new_offline(self.id, username),
            AccountType::Microsoft => Credential::new(
                self.id,
                username,
                AccountType::Microsoft,
                false,
                "access".to_string(),
                "refresh".to_string(),
                self.expires
                    .expect("Microsoft credentials require an expiry in test builder"),
            ),
        };

        if self.active {
            credentials
                .activate()
                .expect("Failed to activate credentials in builder");
        }

        credentials
    }
}
