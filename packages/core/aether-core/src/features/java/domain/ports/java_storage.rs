use async_trait::async_trait;

use crate::features::java::{Java, JavaDomainError};

#[async_trait]
pub trait JavaStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Java>, JavaDomainError>;
    async fn get(&self, version: u32) -> Result<Option<Java>, JavaDomainError>;
    async fn upsert(&self, java: Java) -> Result<Java, JavaDomainError>;
    async fn remove(&self, version: u32) -> Result<(), JavaDomainError>;
}
