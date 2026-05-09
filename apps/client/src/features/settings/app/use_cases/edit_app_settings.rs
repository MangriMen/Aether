use std::sync::Arc;

use crate::{
    core::{WindowLabel, WindowManager},
    features::settings::{
        AppSettings, AppSettingsError, AppSettingsStorage, EditAppSettingsRequest,
    },
};

pub struct EditAppSettingsUseCase<ASS: AppSettingsStorage, WM: WindowManager> {
    app_settings_storage: Arc<ASS>,
    window_manager: Arc<WM>,
}

impl<ASS: AppSettingsStorage, WM: WindowManager> EditAppSettingsUseCase<ASS, WM> {
    pub fn new(app_settings_storage: Arc<ASS>, window_manager: Arc<WM>) -> Self {
        Self {
            app_settings_storage,
            window_manager,
        }
    }

    pub async fn execute(
        &self,
        edit_app_settings: EditAppSettingsRequest,
    ) -> Result<AppSettings, AppSettingsError> {
        let old_settings = self.app_settings_storage.get().await?;
        let mut new_settings = old_settings;

        if let Some(action_on_instance_launch) = edit_app_settings.action_on_instance_launch {
            new_settings.action_on_instance_launch = action_on_instance_launch;
        }

        if let Some(transparent) = edit_app_settings.transparent {
            new_settings.transparent = transparent;
        }

        if let Some(window_effect) = edit_app_settings.window_effect {
            new_settings.window_effect = window_effect;
        }

        self.apply_effect(&new_settings, &old_settings).await?;

        self.app_settings_storage.upsert(new_settings).await?;

        Ok(new_settings)
    }

    async fn apply_effect(
        &self,
        new_settings: &AppSettings,
        old_settings: &AppSettings,
    ) -> Result<(), AppSettingsError> {
        let is_transparent_or_was_it = new_settings.transparent || old_settings.transparent;
        let is_effect_equals = new_settings.window_effect == old_settings.window_effect;

        if is_transparent_or_was_it && !is_effect_equals {
            self.window_manager
                .apply_visual_effect(WindowLabel::Main, new_settings.window_effect)
                .await
                .map_err(|err| AppSettingsError::CanNotSetEffect(err.to_string()))?;
        }

        Ok(())
    }
}
