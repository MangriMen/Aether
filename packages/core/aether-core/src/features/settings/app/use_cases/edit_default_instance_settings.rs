use std::sync::Arc;

use crate::features::settings::{
    DefaultInstanceSettings, DefaultInstanceSettingsStorage, SettingsError,
    app::EditDefaultInstanceSettings,
};

pub struct EditDefaultInstanceSettingsUseCase {
    default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
}

impl EditDefaultInstanceSettingsUseCase {
    pub fn new(default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>) -> Self {
        Self {
            default_instance_settings_storage,
        }
    }

    pub async fn execute(
        &self,
        edit_settings: EditDefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        let mut settings = self.default_instance_settings_storage.get().await?;
        edit_settings.apply_to(&mut settings);
        self.default_instance_settings_storage
            .upsert(settings)
            .await
    }
}
