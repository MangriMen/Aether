use std::sync::Arc;

use crate::features::settings::{
    AppSettings, AppSettingsError, AppSettingsStorage, EditAppSettings, WindowEffect, WindowManager,
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
        edit_app_settings: EditAppSettings,
    ) -> Result<AppSettings, AppSettingsError> {
        let mut settings_state = self.app_settings_storage.get().await?;

        let mut update_app_settings = edit_app_settings;

        if let Some(action_on_instance_launch) = update_app_settings.action_on_instance_launch {
            settings_state.action_on_instance_launch = action_on_instance_launch;
        }

        if let Some(transparent) = update_app_settings.transparent {
            settings_state.transparent = transparent;

            if !transparent {
                update_app_settings.window_effect = Some(WindowEffect::Off);
            }
        }

        if let Some(window_effect) = update_app_settings.window_effect {
            if !settings_state.transparent && window_effect != WindowEffect::Off {
                return Err(AppSettingsError::TransparentEffectIsRequired);
            }

            self.window_manager
                .apply_visual_effects(window_effect)
                .await
                .map_err(|err| AppSettingsError::CanNotSetEffect(err.to_string()))?;

            settings_state.window_effect = window_effect;
        }

        self.app_settings_storage.upsert(settings_state).await?;

        Ok(settings_state)
    }
}
