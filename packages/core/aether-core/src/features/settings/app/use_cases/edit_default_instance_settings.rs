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
        self.default_instance_settings_storage
            .update_mut(Box::new(move |mut settings| {
                let changed = edit_settings.apply_to(&mut settings);
                (settings, changed)
            }))
            .await
    }
}
