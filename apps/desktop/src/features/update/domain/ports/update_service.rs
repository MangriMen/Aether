use crate::features::update::{UpdateError, UpdateStatus};
use async_trait::async_trait;

#[async_trait]
pub trait UpdateService: Send + Sync {
    async fn check(&self) -> Result<Option<UpdateStatus>, UpdateError>;
    async fn install(&self) -> Result<(), UpdateError>;
}
