use async_trait::async_trait;

use crate::{
    features::settings::{DefaultInstanceSettings, Settings, SettingsError},
    shared::UpdateAction,
};

#[async_trait]
pub trait SettingsStorage: Send + Sync {
    async fn get(&self) -> Result<Settings, SettingsError>;
    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError>;
    async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut Settings) -> UpdateAction<R> + Send;
}

#[async_trait]
pub trait DefaultInstanceSettingsStorage: Send + Sync {
    async fn get(&self) -> Result<DefaultInstanceSettings, SettingsError>;

    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError>;

    async fn upsert_with<F, R: Send>(&self, f: F) -> Result<R, SettingsError>
    where
        F: FnOnce(&mut DefaultInstanceSettings) -> UpdateAction<R> + Send;
}
