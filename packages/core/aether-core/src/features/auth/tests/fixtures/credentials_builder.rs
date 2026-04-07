use crate::features::auth::{AccountType, Credentials, Username};
use uuid::Uuid;

/// Builder for constructing test Credentials with customizable fields.
/// Defaults: random UUID, inactive status, Offline account type.
pub struct CredentialsBuilder {
    id: Uuid,
    username: String,
    active: bool,
    account_type: AccountType,
}

impl CredentialsBuilder {
    pub fn new(username: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            active: false,
            account_type: AccountType::Offline,
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

    pub fn build(self) -> Credentials {
        let mut credentials = match self.account_type {
            AccountType::Offline => Credentials::new_offline(
                self.id,
                Username::parse(&self.username).expect("Invalid username in test builder"),
            ),
            AccountType::Microsoft => panic!("Unsupported account type"),
        };

        if self.active {
            credentials
                .activate()
                .expect("Failed to activate credentials in builder");
        }

        credentials
    }
}
