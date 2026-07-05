use std::sync::Arc;

use crate::features::minecraft::{
    MetadataStorage, ModLoader, app::MinecraftApplicationError, modded,
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
        Ok(self
            .metadata_storage
            .get_loader_version_manifest(loader)
            .await?)
    }
}
