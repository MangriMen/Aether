use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::{AuthApplicationError, Credentials};

#[async_trait]
pub trait CredentialsStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Credentials>, AuthApplicationError>;
    async fn get(&self, id: Uuid) -> Result<Credentials, AuthApplicationError>;

    async fn upsert(&self, credentials: Credentials) -> Result<Credentials, AuthApplicationError>;
    async fn upsert_all(
        &self,
        credentials_list: Vec<Credentials>,
    ) -> Result<(), AuthApplicationError>;

    async fn remove(&self, id: Uuid) -> Result<(), AuthApplicationError>;
    async fn clear(&self) -> Result<(), AuthApplicationError>;

    // Queries
    async fn find_active(&self) -> Result<Option<Credentials>, AuthApplicationError>;
}
