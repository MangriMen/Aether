use std::path::Path;

use async_trait::async_trait;

use crate::shared::AssetError;

#[async_trait]
pub trait AssetsStorage: Send + Sync {
    async fn import_file(&self, source: impl AsRef<Path> + Send) -> Result<String, AssetError>;
}
