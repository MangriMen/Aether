use std::sync::Arc;

use crate::features::settings::{Settings, SettingsError, SettingsStorage};

pub struct GetSettingsUseCase {
    settings_storage: Arc<dyn SettingsStorage>,
}

impl GetSettingsUseCase {
    pub fn new(settings_storage: Arc<dyn SettingsStorage>) -> Self {
        Self { settings_storage }
    }

    pub async fn execute(&self) -> Result<Settings, SettingsError> {
        self.settings_storage.get().await
    }
}
