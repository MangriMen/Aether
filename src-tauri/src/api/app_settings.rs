use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    state::{save_settings, ActionOnInstanceLaunch, SettingsState, SettingsStateInner},
    utils::{disable_mica, enable_mica},
    AetherLauncherResult,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppSettings {
    pub action_on_instance_launch: Option<ActionOnInstanceLaunch>,
    pub mica: Option<bool>,
    pub transparent: Option<bool>,
}

#[tauri::command]
pub async fn get_app_settings(
    state: State<'_, SettingsState>,
) -> AetherLauncherResult<SettingsStateInner> {
    let settings_state = state.lock().await;
    Ok(*settings_state)
}

#[tauri::command]
pub async fn update_app_settings(
    app_handle: AppHandle,
    settings_state: State<'_, SettingsState>,
    update_app_settings: UpdateAppSettings,
) -> AetherLauncherResult<()> {
    let mut settings_state = settings_state.lock().await;

    if let Some(action_on_instance_launch) = update_app_settings.action_on_instance_launch {
        settings_state.action_on_instance_launch = action_on_instance_launch;
    }

    let mut need_to_check_mica = true;
    if let Some(transparent) = update_app_settings.transparent {
        settings_state.transparent = transparent;

        if !transparent {
            settings_state.mica = false;

            #[cfg(target_os = "windows")]
            {
                disable_mica(app_handle.clone()).map_err(|e| e.to_string())?;
                settings_state.mica = false;
                need_to_check_mica = false;
            }
        }
    }

    if need_to_check_mica {
        if let Some(mica) = update_app_settings.mica {
            if !settings_state.transparent {
                return Err("Mica can't be turned on without transparent"
                    .to_string()
                    .into());
            }

            #[cfg(target_os = "windows")]
            {
                if mica {
                    enable_mica(app_handle.clone()).map_err(|e| e.to_string())?;
                } else {
                    disable_mica(app_handle.clone()).map_err(|e| e.to_string())?;
                }
                settings_state.mica = mica;
            }
        }
    }

    save_settings(app_handle, &settings_state).await;

    Ok(())
}
