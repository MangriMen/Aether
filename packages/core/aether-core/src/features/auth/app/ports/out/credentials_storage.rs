use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::{AuthApplicationError, Credential};

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
