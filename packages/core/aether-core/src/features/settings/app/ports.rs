use async_trait::async_trait;

use crate::features::settings::{DefaultInstanceSettings, Settings, SettingsError};

#[async_trait]
pub trait SettingsStorage: Send + Sync {
    async fn get(&self) -> Result<Settings, SettingsError>;
    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError>;

    /// Atomically read-modify-write. Returns the final settings.
    /// The closure takes ownership of `Settings` and returns (`modified_settings`, `was_changed`).
    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
    ) -> Result<Settings, SettingsError>;
}

#[async_trait]
pub trait DefaultInstanceSettingsStorage: Send + Sync {
    async fn get(&self) -> Result<DefaultInstanceSettings, SettingsError>;

    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError>;

    /// Atomically read-modify-write. Returns the final settings.
    /// The closure takes ownership of `DefaultInstanceSettings` and returns (`modified`, `was_changed`).
    async fn update_mut(
        &self,
        f: Box<dyn FnOnce(DefaultInstanceSettings) -> (DefaultInstanceSettings, bool) + Send>,
    ) -> Result<DefaultInstanceSettings, SettingsError>;
}
