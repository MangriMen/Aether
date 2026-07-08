use std::sync::Arc;

use async_trait::async_trait;

use crate::features::settings::{
    DefaultInstanceSettings, DefaultInstanceSettingsStorage, SettingsError,
    app::ports::GetDefaultInstanceSettingsUseCasePort,
};

pub struct GetDefaultInstanceSettingsUseCase {
    instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
}

impl GetDefaultInstanceSettingsUseCase {
    pub fn new(instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>) -> Self {
        Self {
            instance_settings_storage,
        }
    }

    pub async fn execute(&self) -> Result<DefaultInstanceSettings, SettingsError> {
        GetDefaultInstanceSettingsUseCasePort::execute(self).await
    }
}

#[async_trait]
impl GetDefaultInstanceSettingsUseCasePort for GetDefaultInstanceSettingsUseCase {
    async fn execute(&self) -> Result<DefaultInstanceSettings, SettingsError> {
        self.instance_settings_storage.get().await
    }
}
