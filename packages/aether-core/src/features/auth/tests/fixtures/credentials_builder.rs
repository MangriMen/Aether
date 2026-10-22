use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::features::auth::domain::{AccountType, Credential, Username};

/// Builder for constructing test Credentials with customizable fields.
/// Defaults: random UUID, inactive status, Offline account type.
pub struct CredentialsBuilder {
    id: Uuid,
    username: String,
    active: bool,
    expired_token: bool,
}

impl CredentialsBuilder {
    pub fn new(username: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            active: false,
            expired_token: false,
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

    /// Builds an account that cannot be activated, which is what an expired
    /// Microsoft token looks like to the domain.
    pub fn with_expired_token(mut self) -> Self {
        self.expired_token = true;
        self
    }

    pub fn build(self) -> Credential {
        let username = Username::parse(&self.username).expect("Invalid username in test builder");

        let mut credentials = if self.expired_token {
            Credential::new(
                self.id,
                username,
                AccountType::Microsoft,
                false,
                "access".to_string(),
                "refresh".to_string(),
                Utc::now() - Duration::hours(1),
            )
        } else {
            Credential::new_offline(self.id, username)
        };

        if self.active {
            credentials
                .activate()
                .expect("Failed to activate credentials in builder");
        }

        credentials
    }
}
