use async_trait::async_trait;

use crate::features::settings::{
    CacheUsage, ClearCacheOutcome, DefaultInstanceSettings, Settings, SettingsError, StorageUsage,
    app::{EditDefaultInstanceSettings, EditSettings},
};

// ── Storage ports ──

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

// ── Use case ports ──

#[async_trait]
pub trait ClearCacheUseCasePort: Send + Sync {
    /// Empty the caches; the app fetches again what it needs. Refused while the app is busy
    /// with something that uses them.
    async fn execute(&self) -> ClearCacheOutcome;
}

#[async_trait]
pub trait GetStorageUsageUseCasePort: Send + Sync {
    async fn execute(&self) -> StorageUsage;
}

#[async_trait]
pub trait GetCacheUsageUseCasePort: Send + Sync {
    async fn execute(&self) -> CacheUsage;
}

#[async_trait]
pub trait IsCacheBusyUseCasePort: Send + Sync {
    /// Whether clearing is refused now. Cheap, unlike measuring the caches.
    async fn execute(&self) -> bool;
}

#[async_trait]
pub trait GetSettingsUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Settings, SettingsError>;
}

#[async_trait]
pub trait GetDefaultInstanceSettingsUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<DefaultInstanceSettings, SettingsError>;
}

#[async_trait]
pub trait EditSettingsUseCasePort: Send + Sync {
    async fn execute(&self, edit_settings: EditSettings) -> Result<Settings, SettingsError>;
}

#[async_trait]
pub trait EditDefaultInstanceSettingsUseCasePort: Send + Sync {
    async fn execute(
        &self,
        edit_settings: EditDefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError>;
}
