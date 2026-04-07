use std::path::Path;

use async_trait::async_trait;

use crate::features::plugins::{ExtractedPlugin, PluginError};

#[async_trait]
pub trait PluginExtractor: Send + Sync {
    async fn extract(&self, file_path: &Path) -> Result<ExtractedPlugin, PluginError>;
}
