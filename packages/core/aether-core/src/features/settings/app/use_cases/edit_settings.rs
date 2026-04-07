use std::sync::Arc;

use crate::{
    features::settings::{app::EditSettings, Settings, SettingsError, SettingsStorage},
    shared::UpdateAction,
};

pub struct EditSettingsUseCase<SS: SettingsStorage> {
    settings_storage: Arc<SS>,
}

impl<SS: SettingsStorage> EditSettingsUseCase<SS> {
    pub fn new(settings_storage: Arc<SS>) -> Self {
        Self { settings_storage }
    }

    pub async fn execute(&self, edit_settings: EditSettings) -> Result<Settings, SettingsError> {
        self.settings_storage
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
