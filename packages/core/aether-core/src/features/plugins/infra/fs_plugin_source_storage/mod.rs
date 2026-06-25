use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        plugins::{PluginError, PluginSource, PluginSourceStorage},
        settings::LocationInfo,
    },
    shared::io::infra::{read_json_async, remove_file, write_json_async},
};

/// Stores `PluginSource` as a `source.json` file alongside each plugin directory.
pub struct FsPluginSourceStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPluginSourceStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_source_path(&self, plugin_id: &str) -> std::path::PathBuf {
        self.location_info.plugin_dir(plugin_id).join("source.json")
    }
}

#[async_trait]
impl PluginSourceStorage for FsPluginSourceStorage {
    async fn save(&self, plugin_id: &str, source: &PluginSource) -> Result<(), PluginError> {
        let path = self.get_source_path(plugin_id);
        write_json_async(&path, source)
            .await
            .map_err(PluginError::Storage)
    }

    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSource>, PluginError> {
        let path = self.get_source_path(plugin_id);

        match read_json_async::<PluginSource>(&path).await {
            Ok(source) => Ok(Some(source)),
            Err(e) => {
                // If file doesn't exist, it's not from GitHub
                if let Some(io_error) = match &e {
                    crate::shared::io::domain::IoError::IoPathError { source, .. }
                    | crate::shared::io::domain::IoError::IoError(source) => Some(source),
                    _ => None,
                } && io_error.kind() == std::io::ErrorKind::NotFound
                {
                    return Ok(None);
                }
                Err(PluginError::Storage(e))
            }
        }
    }

    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError> {
        let path = self.get_source_path(plugin_id);
        remove_file(&path).await.map_err(PluginError::Storage)
    }
}
