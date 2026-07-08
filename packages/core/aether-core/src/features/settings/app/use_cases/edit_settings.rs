use std::sync::Arc;

use async_trait::async_trait;

use crate::features::settings::{
    Settings, SettingsError, SettingsStorage,
    app::{EditSettings, ports::EditSettingsUseCasePort},
};

pub struct EditSettingsUseCase {
    settings_storage: Arc<dyn SettingsStorage>,
}

impl EditSettingsUseCase {
    pub fn new(settings_storage: Arc<dyn SettingsStorage>) -> Self {
        Self { settings_storage }
    }

    pub async fn execute(&self, edit_settings: EditSettings) -> Result<Settings, SettingsError> {
        EditSettingsUseCasePort::execute(self, edit_settings).await
    }
}

#[async_trait]
impl EditSettingsUseCasePort for EditSettingsUseCase {
    async fn execute(&self, edit_settings: EditSettings) -> Result<Settings, SettingsError> {
        self.settings_storage
            .update_mut(Box::new(move |mut settings| {
                let changed = edit_settings.apply_to(&mut settings);
                (settings, changed)
            }))
            .await
    }
}
