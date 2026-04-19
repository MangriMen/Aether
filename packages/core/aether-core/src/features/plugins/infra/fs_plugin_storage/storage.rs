use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

use crate::{
    features::{
        plugins::{
            ExtractedPlugin, LoadConfig, Plugin, PluginCapabilities, PluginContent, PluginError,
            PluginManifest, PluginState, PluginStorage,
        },
        settings::LocationInfo,
    },
    shared::{
        IoError, copy_dir_all, create_dir_all, read_async, read_dir, read_json_async,
        remove_dir_all, sha1_async,
    },
};

use super::FsPluginStorageConstants;

pub struct FsPluginStorage {
    location_info: Arc<LocationInfo>,
    constants: FsPluginStorageConstants,
}

impl FsPluginStorage {
    pub fn new(
        location_info: Arc<LocationInfo>,
        constants: Option<FsPluginStorageConstants>,
    ) -> Self {
        Self {
            location_info,
            constants: constants.unwrap_or_default(),
        }
    }

    fn get_manifest_path(&self, dir: &Path) -> PathBuf {
        dir.join(self.constants.manifest_filename)
    }

    fn get_capabilities_path(&self, dir: &Path) -> PathBuf {
        dir.join(self.constants.capabilities_filename)
    }

    async fn load_manifest(&self, dir: &Path) -> Result<PluginManifest, PluginError> {
        Ok(read_json_async(&self.get_manifest_path(dir)).await?)
    }

    async fn load_capabilities(
        &self,
        dir: &Path,
    ) -> Result<Option<PluginCapabilities>, PluginError> {
        Ok(read_json_async(&self.get_capabilities_path(dir))
            .await
            .map(Some)
            .or_else(|e| {
                if let Some(io_error) = match &e {
                    IoError::IoPathError { source, .. } | IoError::IoError(source) => Some(source),
                    _ => None,
                } && io_error.kind() == std::io::ErrorKind::NotFound
                {
                    return Ok(None);
                }
                Err(e)
            })?)
    }

    async fn calc_hash(dir: &Path, manifest: &PluginManifest) -> Result<String, PluginError> {
        let relative_file_path = match manifest.load.clone() {
            LoadConfig::Extism { file, .. } => file,
            LoadConfig::Native { lib_path } => lib_path,
        };

        let absolute_file_path = dir.join(relative_file_path);
        let file_content = read_async(&absolute_file_path).await?;

        Ok(sha1_async(file_content).await)
    }

    async fn load_from_dir(&self, dir: &Path) -> Result<Plugin, PluginError> {
        let manifest = self.load_manifest(dir).await?;
        let capabilities = self.load_capabilities(dir).await?;
        let hash = Self::calc_hash(dir, &manifest).await?;

        Ok(Plugin {
            manifest,
            capabilities,
            hash,
            state: PluginState::NotLoaded,
        })
    }
}

#[async_trait]
impl PluginStorage for FsPluginStorage {
    async fn add(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError> {
        let source_dir = match &extracted_plugin.content {
            PluginContent::Filesystem { temp_dir } => temp_dir,
        };

        let target_dir = self.location_info.plugin_dir(&extracted_plugin.plugin_id);
        create_dir_all(&target_dir).await?;

        copy_dir_all(source_dir, target_dir)?;

        Ok(())
    }

    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError> {
        let plugins_dir = self.location_info.plugins_dir();

        if !plugins_dir.exists() {
            create_dir_all(&plugins_dir).await?;
        }

        let mut dir_entries = read_dir(&plugins_dir).await?;
        let mut plugins = HashMap::new();

        while let Some(dir_entry) = dir_entries.next_entry().await.map_err(IoError::from)? {
            let plugin_dir = dir_entry.path();

            match self.load_from_dir(&plugin_dir).await {
                Ok(plugin) => {
                    plugins.insert(plugin.manifest.metadata.id.clone(), plugin);
                }
                Err(e) => {
                    log::debug!(
                        "Failed to load plugin from '{}': {}",
                        plugin_dir.display(),
                        e
                    );
                }
            }
        }

        Ok(plugins)
    }

    async fn get(&self, plugin_id: &str) -> Result<Plugin, PluginError> {
        let plugin_dir = self.location_info.plugin_dir(plugin_id);
        self.load_from_dir(&plugin_dir).await
    }

    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError> {
        let plugin_dir = self.location_info.plugin_dir(plugin_id);

        if !plugin_dir.exists() {
            return Ok(());
        }

        remove_dir_all(plugin_dir).await?;

        Ok(())
    }
}
