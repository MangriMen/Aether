use std::sync::Arc;

use crate::features::minecraft::{
    MetadataStorage, MinecraftApplicationError, VersionManifestService, vanilla,
};

pub struct GetVersionManifestUseCase {
    metadata_storage: Arc<dyn MetadataStorage>,
}

#[async_trait::async_trait]
impl VersionManifestService for GetVersionManifestUseCase {
    async fn execute(&self) -> Result<vanilla::VersionManifest, MinecraftApplicationError> {
        Ok(self.metadata_storage.get_version_manifest().await?)
    }
}

impl GetVersionManifestUseCase {
    pub fn new(metadata_storage: Arc<dyn MetadataStorage>) -> Self {
        Self { metadata_storage }
    }

    pub async fn execute(&self) -> Result<vanilla::VersionManifest, MinecraftApplicationError> {
        VersionManifestService::execute(self).await
    }
}
