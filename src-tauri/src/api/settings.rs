use aether_core::features::settings::Settings;
use tauri::{utils::config::WindowEffectsConfig, window::Effect, Manager, State};

use crate::{
    state::{save_settings, ActionOnInstanceLaunch, SettingsState},
    AetherLauncherResult,
};

#[tauri::command]
pub async fn get_action_on_instance_launch(
    state: State<'_, SettingsState>,
) -> AetherLauncherResult<ActionOnInstanceLaunch> {
    let settings_state = state.lock().await;

    Ok(settings_state.action_on_instance_launch)
}

#[tauri::command]
pub async fn set_action_on_instance_launch(
    app_handle: tauri::AppHandle,
    state: State<'_, SettingsState>,
    action_on_instance_launch: ActionOnInstanceLaunch,
) -> Result<(), ()> {
    let mut settings_state = state.lock().await;

    settings_state.action_on_instance_launch = action_on_instance_launch;

    save_settings(app_handle, &settings_state).await;

    Ok(())
}

#[tauri::command]
pub async fn get_mica(state: State<'_, SettingsState>) -> AetherLauncherResult<bool> {
    let settings_state = state.lock().await;

    Ok(settings_state.mica)
}

#[tauri::command]
pub async fn set_mica(
    app_handle: tauri::AppHandle,
    state: State<'_, SettingsState>,
    enabled: bool,
) -> Result<(), String> {
    let mut settings_state = state.lock().await;

    #[cfg(target_os = "windows")]
    {
        if let Some(main_window) = app_handle.get_webview_window("main") {
            if enabled {
                let _ = main_window.set_effects(WindowEffectsConfig {
                    effects: vec![Effect::Mica],
                    state: Some(tauri::window::EffectState::FollowsWindowActiveState),
                    radius: None,
                    color: None,
                });
            } else {
                let _ = main_window.set_effects(None);
            }

            settings_state.mica = enabled;

            save_settings(app_handle, &settings_state).await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_settings() -> AetherLauncherResult<Settings> {
    Ok(aether_core::api::settings::get().await?)
}

#[tauri::command]
pub async fn get_max_ram() -> AetherLauncherResult<u64> {
    Ok(aether_core::shared::infra::get_total_memory())
}
