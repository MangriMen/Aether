use std::path::Path;

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsError, SettingsStorage},
    shared::{IoError, JsonValueStore, UpdateAction},
};

pub struct FsSettingsStorage {
    store: JsonValueStore<Settings>,
}

impl FsSettingsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            store: JsonValueStore::new(settings_dir.join("settings.json")),
        }
    }
}

#[async_trait]
impl SettingsStorage for FsSettingsStorage {
    async fn get(&self) -> Result<Settings, SettingsError> {
        self.store.read().await.map_err(|e| match e {
            IoError::IoPathError { ref source, .. } | IoError::IoError(ref source) => {
                match source.kind() {
                    std::io::ErrorKind::NotFound => SettingsError::NotFound,
                    _ => SettingsError::StorageFailure(e),
                }
            }
            IoError::SerializationError(_) | IoError::DeserializationError(_) => {
                SettingsError::StorageFailure(e)
            }
        })
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
        self.store.write(&settings).await?;
        Ok(settings)
    }

    async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut Settings) -> UpdateAction<R> + Send,
    {
        self.store.update(f).await.map_err(|e| match e {
            IoError::IoPathError { ref source, .. } | IoError::IoError(ref source) => {
                match source.kind() {
                    std::io::ErrorKind::NotFound => SettingsError::NotFound,
                    _ => SettingsError::StorageFailure(e),
                }
            }
            IoError::SerializationError(_) | IoError::DeserializationError(_) => {
                SettingsError::StorageFailure(e)
            }
        })
    }
}
