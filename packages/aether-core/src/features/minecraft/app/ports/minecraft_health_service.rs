use async_trait::async_trait;

use crate::features::minecraft::{MinecraftApplicationError, MinecraftHealthParams};

#[async_trait]
pub trait MinecraftHealthService: Send + Sync {
    async fn verify_files(
        &self,
        params: MinecraftHealthParams,
    ) -> Result<bool, MinecraftApplicationError>;
}
