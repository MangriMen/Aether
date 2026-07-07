use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::settings::{
        DefaultInstanceSettings, DefaultInstanceSettingsStorage, SettingsError,
        infra::fs::models::DefaultInstanceSettingsV1,
    },
    shared::json_store::{domain::UpdateAction, infra::JsonValueStore},
};

pub struct FsDefaultInstanceSettingsStorage {
    store: JsonValueStore<DefaultInstanceSettingsV1>,
    path: PathBuf,
}

impl FsDefaultInstanceSettingsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        let path = settings_dir.join("instance_settings.json");
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
impl DefaultInstanceSettingsStorage for FsDefaultInstanceSettingsStorage {
    async fn get(&self) -> Result<DefaultInstanceSettings, SettingsError> {
        Ok(self.store.read_or_default().await?.into())
    }

    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        self.store
            .write(&DefaultInstanceSettingsV1::from(&settings))
            .await?;
        Ok(settings)
    }

    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(DefaultInstanceSettings) -> (DefaultInstanceSettings, bool) + Send>,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        let settings: DefaultInstanceSettings = self
            .store
            .update_with_default(move |dto| {
                let domain_settings = DefaultInstanceSettings::from(dto.clone());
                let (domain_settings, changed) = f(domain_settings);
                if changed {
                    *dto = DefaultInstanceSettingsV1::from(&domain_settings);
                    UpdateAction::Save(domain_settings)
                } else {
                    UpdateAction::NoChanges(domain_settings)
                }
            })
            .await?;
        Ok(settings)
    }
}

impl FsDefaultInstanceSettingsStorage {
    /// Same signature as trait but with boxed closure for object-safety.
    pub async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut DefaultInstanceSettings) -> UpdateAction<R> + Send,
    {
        Ok(self
            .store
            .update_with_default(|dto| {
                let mut domain_settings = DefaultInstanceSettings::from(dto.clone());

                let action = f(&mut domain_settings);

                match action {
                    UpdateAction::Save(return_value) => {
                        *dto = DefaultInstanceSettingsV1::from(&domain_settings);
                        UpdateAction::Save(return_value)
                    }
                    UpdateAction::NoChanges(return_value) => UpdateAction::NoChanges(return_value),
                }
            })
            .await?)
    }
}
