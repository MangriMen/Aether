use async_trait::async_trait;

use crate::features::minecraft::{MinecraftApplicationError, ModLoader, modded};

/// Port trait for the get loader version manifest use case.
#[async_trait]
pub trait GetLoaderVersionManifestUseCasePort: Send + Sync {
    async fn execute(
        &self,
        loader: ModLoader,
    ) -> Result<modded::Manifest, MinecraftApplicationError>;
}
