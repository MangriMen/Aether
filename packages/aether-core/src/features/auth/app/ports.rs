use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::{
    app::{Account, AuthApplicationError},
    domain::Credential,
};

#[async_trait]
pub trait CredentialsStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Credential>, AuthApplicationError>;
    async fn get(&self, id: Uuid) -> Result<Credential, AuthApplicationError>;

    async fn upsert(&self, credentials: Credential) -> Result<Credential, AuthApplicationError>;
    async fn upsert_all(
        &self,
        credentials_list: Vec<Credential>,
    ) -> Result<(), AuthApplicationError>;

    async fn remove(&self, id: Uuid) -> Result<(), AuthApplicationError>;
    async fn clear(&self) -> Result<(), AuthApplicationError>;

    // Queries
    async fn find_active(&self) -> Result<Option<Credential>, AuthApplicationError>;
}

// ── Use case ports ──

#[async_trait]
pub trait CreateOfflineAccountUseCasePort: Send + Sync {
    async fn execute(&self, username: String) -> Result<Account, AuthApplicationError>;
}

#[async_trait]
pub trait GetAccountsUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<Account>, AuthApplicationError>;
}

#[async_trait]
pub trait LogoutUseCasePort: Send + Sync {
    async fn execute(&self, account_id: Uuid) -> Result<(), AuthApplicationError>;
}

#[async_trait]
pub trait SetActiveAccountUseCasePort: Send + Sync {
    async fn execute(&self, account_id: Uuid) -> Result<Account, AuthApplicationError>;
}
