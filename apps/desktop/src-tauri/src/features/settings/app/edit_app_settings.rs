use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    features::settings::{
        save_settings, set_window_effect, ActionOnInstanceLaunch, AppSettingsError,
        AppSettingsState, WindowEffect,
    },
    FrontendResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditAppSettings {
    pub action_on_instance_launch: Option<ActionOnInstanceLaunch>,
    pub transparent: Option<bool>,
    pub window_effect: Option<WindowEffect>,
}

pub async fn edit_app_settings_use_case<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    settings_state: State<'_, AppSettingsState>,
    edit_app_settings: EditAppSettings,
) -> FrontendResult<()> {
    let mut settings_state = settings_state.lock().await;

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
            return Err(crate::Error::AppSettingsError(
                AppSettingsError::TransparentEffectIsRequired,
            )
            .into());
        }
        set_window_effect(app_handle.clone(), window_effect).map_err(|e| {
            crate::Error::AppSettingsError(AppSettingsError::CanNotSetEffect(e.to_string()))
        })?;
        settings_state.window_effect = window_effect;
    }

    save_settings(app_handle, &settings_state).await;

    Ok(())
}
