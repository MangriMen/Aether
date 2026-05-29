use std::path::PathBuf;

use aether_core::shared::json_store::infra::JsonValueStore;
use async_trait::async_trait;
use log::warn;

use crate::features::settings::{AppSettings, AppSettingsError, AppSettingsStorage};

pub struct FsAppSettingsStorage {
    store: JsonValueStore<AppSettings>,
    path: PathBuf,
}

impl FsAppSettingsStorage {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            store: JsonValueStore::new(path.clone()),
            path,
        }
    }

    pub fn get_file_path(&self) -> PathBuf {
        self.path.clone()
    }
}

#[async_trait]
impl AppSettingsStorage for FsAppSettingsStorage {
    async fn get(&self) -> Result<AppSettings, AppSettingsError> {
        Ok(self.store.read_or_default().await.unwrap_or_else(|err| {
            warn!("Failed to read settings: {err}");

            AppSettings::default()
        }))
    }

    async fn upsert(&self, settings: AppSettings) -> Result<(), AppSettingsError> {
        Ok(self
            .store
            .write(&settings)
            .await
            .map_err(|err| AppSettingsError::Storage(err.to_string()))?)
    }
}
