use std::sync::Arc;

use crate::features::settings::{
    DefaultInstanceSettings, DefaultInstanceSettingsStorage, SettingsError,
};

pub struct GetDefaultInstanceSettingsUseCase<SS: DefaultInstanceSettingsStorage> {
    instance_settings_storage: Arc<SS>,
}

impl<SS: DefaultInstanceSettingsStorage> GetDefaultInstanceSettingsUseCase<SS> {
    pub fn new(instance_settings_storage: Arc<SS>) -> Self {
        Self {
            instance_settings_storage,
        }
    }

    pub async fn execute(&self) -> Result<DefaultInstanceSettings, SettingsError> {
        self.instance_settings_storage.get().await
    }
}
