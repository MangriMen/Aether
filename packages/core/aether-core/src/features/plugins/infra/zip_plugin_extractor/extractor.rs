use std::{io::Cursor, path::Path};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::plugins::{
        ExtractedPlugin, PluginContent, PluginError, PluginExtractor, PluginManifest,
    },
    shared::{read_async, IoError},
};

use super::ZipPluginExtractorConstants;

#[derive(Default)]
pub struct ZipPluginExtractor {
    constants: ZipPluginExtractorConstants,
}

impl ZipPluginExtractor {
    pub fn new(constants: ZipPluginExtractorConstants) -> Self {
        Self { constants }
    }

    async fn read_manifest(
        &self,
        archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
    ) -> Result<PluginManifest, PluginError> {
        let manifest_file = archive
            .by_name(self.constants.manifest_filename)
            .map_err(|_| PluginError::ManifestNotFound {
                path: self.constants.manifest_filename.to_string(),
            })?;

        serde_json::from_reader(manifest_file).map_err(|e| PluginError::InvalidManifestFormat {
            error: e.to_string(),
        })
    }
}

#[async_trait]
impl PluginExtractor for ZipPluginExtractor {
    async fn extract(&self, file_path: &Path) -> Result<ExtractedPlugin, PluginError> {
        let source_path = file_path.to_string_lossy().to_string();

        let file = read_async(file_path)
            .await
            .map_err(|_| PluginError::ExtractionFailed {
                from: source_path.clone(),
            })?;

        let mut archive = zip::ZipArchive::new(Cursor::new(file))
            .map_err(|_| PluginError::InvalidExtractionFormat)?;

        let manifest = self.read_manifest(&mut archive).await?;
        let plugin_id = manifest.metadata.id.clone();

        let temp_dir = TempDir::new().map_err(IoError::from)?;
        archive
            .extract(&temp_dir)
            .map_err(|_| PluginError::FileExtractionFailed { from: source_path })?;

        Ok(ExtractedPlugin {
            plugin_id,
            manifest,
            content: PluginContent::Filesystem { temp_dir },
        })
    }
}
