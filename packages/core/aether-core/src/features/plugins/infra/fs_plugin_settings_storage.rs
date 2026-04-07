use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    features::{
        plugins::{PluginError, PluginSettings, PluginSettingsStorage},
        settings::LocationInfo,
    },
    shared::{read_toml_async, write_toml_async},
};

pub struct FsPluginSettingsStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPluginSettingsStorage {
    pub async fn read<T>(&self, path: &Path) -> Result<Option<T>, PluginError>
    where
        T: DeserializeOwned,
    {
        if !path.exists() {
            return Ok(None);
        }

        let value = read_toml_async::<T>(path).await?;

        Ok(Some(value))
    }

    pub async fn write<T>(&self, path: &Path, value: &T) -> Result<(), PluginError>
    where
        T: Serialize,
    {
        Ok(write_toml_async(path, value).await?)
    }
}

impl FsPluginSettingsStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_plugin_settings_path(&self, plugin_id: &str) -> PathBuf {
        self.location_info.plugin_settings(plugin_id)
    }
}

#[async_trait]
impl PluginSettingsStorage for FsPluginSettingsStorage {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
        self.read(&self.get_plugin_settings_path(plugin_id)).await
    }

    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), PluginError> {
        self.write(&self.get_plugin_settings_path(plugin_id), settings)
            .await
    }
}
