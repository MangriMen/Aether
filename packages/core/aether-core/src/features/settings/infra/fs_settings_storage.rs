use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsError, SettingsStorage},
    shared::{IoError, JsonValueStore, UpdateAction},
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
}

#[async_trait]
impl SettingsStorage for FsSettingsStorage {
    async fn get(&self) -> Result<Settings, SettingsError> {
        Ok(self.store.read().await?)
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
