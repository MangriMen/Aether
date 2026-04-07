use std::sync::Arc;

use crate::{
    features::settings::{
        app::EditDefaultInstanceSettings, DefaultInstanceSettings, DefaultInstanceSettingsStorage,
        SettingsError,
    },
    shared::UpdateAction,
};

pub struct EditDefaultInstanceSettingsUseCase<DISS: DefaultInstanceSettingsStorage> {
    default_instance_settings_storage: Arc<DISS>,
}

impl<DIS: DefaultInstanceSettingsStorage> EditDefaultInstanceSettingsUseCase<DIS> {
    pub fn new(default_instance_settings_storage: Arc<DIS>) -> Self {
        Self {
            default_instance_settings_storage,
        }
    }

    pub async fn execute(
        &self,
        edit_settings: EditDefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        self.default_instance_settings_storage
            .upsert_with(|settings| {
                if edit_settings.apply_to(settings) {
                    UpdateAction::Save(settings.to_owned())
                } else {
                    UpdateAction::NoChanges(settings.to_owned())
                }
            })
            .await
    }
}
