use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsError},
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
