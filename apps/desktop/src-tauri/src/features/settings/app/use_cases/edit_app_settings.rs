use std::sync::Arc;

use crate::features::settings::{
    AppSettings, AppSettingsError, AppSettingsStorage, EditAppSettingsDto, WindowEffectDto,
    WindowManager,
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
        edit_app_settings: EditAppSettingsDto,
    ) -> Result<AppSettings, AppSettingsError> {
        let mut new_settings = self.app_settings_storage.get().await?;

        let mut update_app_settings = edit_app_settings;

        if let Some(action_on_instance_launch) = update_app_settings.action_on_instance_launch {
            new_settings.action_on_instance_launch = action_on_instance_launch.into();
        }

        if let Some(transparent) = update_app_settings.transparent {
            new_settings.transparent = transparent;

            if !transparent {
                update_app_settings.window_effect = Some(WindowEffectDto::Off);
            }
        }

        if let Some(window_effect) = update_app_settings.window_effect {
            if !new_settings.transparent && window_effect != WindowEffectDto::Off {
                return Err(AppSettingsError::TransparentEffectRequired);
            }

            self.window_manager
                .apply_visual_effects(window_effect.into())
                .await
                .map_err(|err| AppSettingsError::CanNotSetEffect(err.clone()))?;

            new_settings.window_effect = window_effect.into();
        }

        self.app_settings_storage.upsert(new_settings).await?;

        Ok(new_settings)
    }
}
