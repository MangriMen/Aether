use std::sync::Arc;

use crate::features::settings::{Settings, SettingsError, SettingsStorage};

pub struct GetSettingsUseCase<SS: SettingsStorage> {
    settings_storage: Arc<SS>,
}

impl<SS: SettingsStorage> GetSettingsUseCase<SS> {
    pub fn new(settings_storage: Arc<SS>) -> Self {
        Self { settings_storage }
    }

    pub async fn execute(&self) -> Result<Settings, SettingsError> {
        self.settings_storage.get().await
    }
}
