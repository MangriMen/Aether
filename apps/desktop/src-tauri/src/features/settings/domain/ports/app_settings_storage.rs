use async_trait::async_trait;

use crate::features::settings::{AppSettings, AppSettingsError};

#[async_trait]
pub trait AppSettingsStorage: Send + Sync {
    async fn get(&self) -> Result<AppSettings, AppSettingsError>;
    async fn upsert(&self, settings: AppSettings) -> Result<(), AppSettingsError>;
}
