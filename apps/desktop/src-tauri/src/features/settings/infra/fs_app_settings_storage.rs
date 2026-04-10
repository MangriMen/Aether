use std::path::PathBuf;

use aether_core::shared::JsonValueStore;
use async_trait::async_trait;
use log::warn;

use crate::features::settings::{AppSettings, AppSettingsError, AppSettingsStorage};

pub struct FsAppSettingsStorage {
    store: JsonValueStore<AppSettings>,
}

impl FsAppSettingsStorage {
    pub fn new(path: PathBuf) -> Self {
        Self {
            store: JsonValueStore::new(path),
        }
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
            .map_err(|_| AppSettingsError::SaveError)?)
    }
}
