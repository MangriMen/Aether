use std::sync::Arc;

use crate::features::update::{TauriUpdateService, UpdateStatus};
use async_trait::async_trait;

#[async_trait]
pub trait UpdateService: Send + Sync {
    async fn check(&self) -> Result<Option<UpdateStatus>, String>;
    async fn install(&self) -> Result<(), String>;
}

pub type UpdateServiceState<R> = Arc<TauriUpdateService<R>>;
