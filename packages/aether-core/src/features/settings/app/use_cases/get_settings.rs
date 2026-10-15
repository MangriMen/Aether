use std::sync::Arc;

use async_trait::async_trait;

use crate::features::settings::{
    Settings, SettingsError, SettingsStorage, app::ports::GetSettingsUseCasePort,
};

pub struct GetSettingsUseCase {
    settings_storage: Arc<dyn SettingsStorage>,
}

impl GetSettingsUseCase {
    pub fn new(settings_storage: Arc<dyn SettingsStorage>) -> Self {
        Self { settings_storage }
    }
}

#[async_trait]
impl GetSettingsUseCasePort for GetSettingsUseCase {
    async fn execute(&self) -> Result<Settings, SettingsError> {
        self.settings_storage.get().await
    }
}
