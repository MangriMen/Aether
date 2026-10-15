use async_trait::async_trait;

use crate::features::minecraft::{MinecraftApplicationError, vanilla};

#[async_trait]
pub trait VersionManifestService: Send + Sync {
    async fn execute(&self) -> Result<vanilla::VersionManifest, MinecraftApplicationError>;
}
