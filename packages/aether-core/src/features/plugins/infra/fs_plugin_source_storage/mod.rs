use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::PluginId,
        plugins::{PluginError, PluginSource, PluginSourceStorage},
        settings::LocationInfo,
    },
    shared::io::{
        domain::IoError,
        infra::{read_json_async, write_json_async},
    },
};

pub(super) const SOURCE_FILE_NAME: &str = "source.json";

/// Stores `PluginSource` as a `source.json` file alongside each plugin directory.
pub struct FsPluginSourceStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPluginSourceStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_source_path(&self, plugin_id: &PluginId) -> std::path::PathBuf {
        self.location_info
            .plugin_dir(plugin_id)
            .join(SOURCE_FILE_NAME)
    }
}

#[async_trait]
impl PluginSourceStorage for FsPluginSourceStorage {
    async fn save(&self, plugin_id: &PluginId, source: &PluginSource) -> Result<(), PluginError> {
        let path = self.get_source_path(plugin_id);
        write_json_async(&path, source)
            .await
            .map_err(PluginError::Storage)
    }

    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginSource>, PluginError> {
        let path = self.get_source_path(plugin_id);

        match read_json_async::<PluginSource>(&path).await {
            Ok(source) => Ok(Some(source)),
            Err(e) => {
                // If file doesn't exist, there's no remote source configured
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

    /// No record is already the state asked for.
    async fn remove(&self, plugin_id: &PluginId) -> Result<(), PluginError> {
        let path = self.get_source_path(plugin_id);
        match tokio::fs::remove_file(&path).await {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            result => result.map_err(|err| PluginError::Storage(IoError::with_path(err, &path))),
        }
    }
}
