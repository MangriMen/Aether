use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tracing::warn;

use crate::{
    features::settings::{
        Settings, SettingsError, SettingsStorage, infra::fs::settings::SettingsV2,
    },
    shared::{
        io::{domain::IoError, infra::read_json_async},
        json_store::{domain::UpdateAction, infra::JsonValueStore},
    },
};

pub struct FsSettingsStorage {
    store: JsonValueStore<SettingsV2>,
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
            Ok(settings) => Ok(settings.into()),
            Err(original_err) => {
                match read_json_async::<super::settings::SettingsV1>(self.path.clone()).await {
                    Ok(v1) => {
                        warn!("Migrating instance from V1 at {:?}", self.path.clone());
                        let migrated: Settings = v1.into();
                        let v2 = SettingsV2::from(&migrated);

                        if let Err(e) = self.store.write(&v2).await {
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
        self.store.write(&(&settings).into()).await?;
        Ok(settings)
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
    ) -> Result<Settings, SettingsError> {
        let result: Settings = self
            .store
            .update(move |dto| {
                let settings = Settings::from(dto.clone());
                let (settings, changed) = f(settings);
                if changed {
                    *dto = SettingsV2::from(&settings);
                    UpdateAction::Save(settings)
                } else {
                    UpdateAction::NoChanges(settings)
                }
            })
            .await?;
        Ok(result)
    }
}

impl FsSettingsStorage {
    /// Same signature as `SettingsStorage` trait but with boxed closure for object-safety.
    pub async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut Settings) -> UpdateAction<R> + Send,
    {
        Ok(self
            .store
            .update(|dto| {
                let mut settings = Settings::from(dto.clone());

                let action = f(&mut settings);

                match action {
                    UpdateAction::Save(return_value) => {
                        *dto = SettingsV2::from(&settings);
                        UpdateAction::Save(return_value)
                    }
                    UpdateAction::NoChanges(return_value) => UpdateAction::NoChanges(return_value),
                }
            })
            .await?)
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
