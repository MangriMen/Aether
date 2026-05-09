use crate::features::update::UpdateStatus;
use async_trait::async_trait;

#[async_trait]
pub trait UpdateService: Send + Sync {
    async fn check(&self) -> Result<Option<UpdateStatus>, String>;
    async fn install(&self) -> Result<(), String>;
}
