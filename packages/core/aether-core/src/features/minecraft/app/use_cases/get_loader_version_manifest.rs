use std::sync::Arc;

use async_trait::async_trait;

use crate::features::minecraft::{
    MetadataStorage, ModLoader, app::MinecraftApplicationError,
    app::ports::GetLoaderVersionManifestUseCasePort, modded,
};

pub struct GetLoaderVersionManifestUseCase {
    metadata_storage: Arc<dyn MetadataStorage>,
}

impl GetLoaderVersionManifestUseCase {
    pub fn new(metadata_storage: Arc<dyn MetadataStorage>) -> Self {
        Self { metadata_storage }
    }

    pub async fn execute(
        &self,
        loader: ModLoader,
    ) -> Result<modded::Manifest, MinecraftApplicationError> {
        GetLoaderVersionManifestUseCasePort::execute(self, loader).await
    }
}

#[async_trait]
impl GetLoaderVersionManifestUseCasePort for GetLoaderVersionManifestUseCase {
    async fn execute(
        &self,
        loader: ModLoader,
    ) -> Result<modded::Manifest, MinecraftApplicationError> {
        Ok(self
            .metadata_storage
            .get_loader_version_manifest(loader)
            .await?)
    }
}
