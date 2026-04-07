use std::path::Path;

use async_trait::async_trait;

use crate::{
    features::settings::{DefaultInstanceSettings, DefaultInstanceSettingsStorage, SettingsError},
    shared::{JsonValueStore, UpdateAction},
};

pub struct FsDefaultInstanceSettingsStorage {
    store: JsonValueStore<DefaultInstanceSettings>,
}

impl FsDefaultInstanceSettingsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            store: JsonValueStore::new(settings_dir.join("instance_settings.json")),
        }
    }
}

#[async_trait]
impl DefaultInstanceSettingsStorage for FsDefaultInstanceSettingsStorage {
    async fn get(&self) -> Result<DefaultInstanceSettings, SettingsError> {
        Ok(self.store.read_or_default().await?)
    }

    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        self.store.write(&settings).await?;
        Ok(settings)
    }

    async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut DefaultInstanceSettings) -> UpdateAction<R> + Send,
    {
        Ok(self.store.update_with_default(f).await?)
    }
}
