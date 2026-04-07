use async_trait::async_trait;

use crate::features::minecraft::{MinecraftDomainError, ModLoader};

#[async_trait]
pub trait MetadataStorage: Send + Sync {
    async fn get_version_manifest(
        &self,
    ) -> Result<daedalus::minecraft::VersionManifest, MinecraftDomainError>;
    async fn get_loader_version_manifest(
        &self,
        loader: ModLoader,
    ) -> Result<daedalus::modded::Manifest, MinecraftDomainError>;
}
