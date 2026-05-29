use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tracing::warn;

use crate::{
    features::settings::{Settings, SettingsError, SettingsStorage},
    shared::{
        io::domain::IoError,
        io::infra::read_json_async,
        json_store::{domain::UpdateAction, infra::JsonValueStore},
    },
};

pub struct FsSettingsStorage {
    store: JsonValueStore<Settings>,
    path: PathBuf,
}

impl FsSettingsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        let path = settings_dir.join("settings.json");
        Self {
            store: JsonValueStore::new(path.clone()),
            path,
        }
    }

    pub fn get_file_path(&self) -> PathBuf {
        self.path.clone()
    }

    async fn load_settings_internal(&self) -> Result<Settings, SettingsError> {
        match self.store.read().await {
            Ok(instance) => Ok(instance),
            Err(original_err) => {
                match read_json_async::<super::settings::SettingsV1>(self.path.clone()).await {
                    Ok(v1) => {
                        warn!("Migrating instance from V1 at {:?}", self.path.clone());
                        let migrated: Settings = v1.into();

                        if let Err(e) = self.store.write(&migrated).await {
                            warn!("Failed to save migrated instance: {:?}", e);
                        }

                        Ok(migrated)
                    }
                    Err(_) => Err(SettingsError::Storage(original_err.to_string())),
                }
            }
        }
    }
}

#[async_trait]
impl SettingsStorage for FsSettingsStorage {
    async fn get(&self) -> Result<Settings, SettingsError> {
        self.load_settings_internal().await
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
        self.store.write(&settings).await?;
        Ok(settings)
    }

    async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut Settings) -> UpdateAction<R> + Send,
    {
        Ok(self.store.update(f).await?)
    }
}

impl From<IoError> for SettingsError {
    fn from(value: IoError) -> Self {
        match value {
            IoError::IoPathError { ref source, .. } | IoError::IoError(ref source) => {
                match source.kind() {
                    std::io::ErrorKind::NotFound => SettingsError::NotFound,
                    _ => SettingsError::Storage(value.to_string()),
                }
            }
            _ => SettingsError::Storage(value.to_string()),
        }
    }
}
