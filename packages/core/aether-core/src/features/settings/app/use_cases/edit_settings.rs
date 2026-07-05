use std::sync::Arc;

use crate::features::settings::{Settings, SettingsError, SettingsStorage, app::EditSettings};

pub struct EditSettingsUseCase {
    settings_storage: Arc<dyn SettingsStorage>,
}

impl EditSettingsUseCase {
    pub fn new(settings_storage: Arc<dyn SettingsStorage>) -> Self {
        Self { settings_storage }
    }

    pub async fn execute(&self, edit_settings: EditSettings) -> Result<Settings, SettingsError> {
        let mut settings = self.settings_storage.get().await?;
        edit_settings.apply_to(&mut settings);
        self.settings_storage.upsert(settings).await
    }
}
