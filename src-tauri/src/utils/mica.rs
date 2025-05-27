use tauri::{utils::config::WindowEffectsConfig, window::Effect, AppHandle, Manager};

pub fn enable_mica(app_handle: AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    {
        if let Some(main_window) = app_handle.get_webview_window("main") {
            main_window.set_effects(WindowEffectsConfig {
                effects: vec![Effect::Mica],
                state: Some(tauri::window::EffectState::FollowsWindowActiveState),
                radius: None,
                color: None,
            })?;
        }
    }

    Ok(())
}

pub fn disable_mica(app_handle: AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    {
        if let Some(main_window) = app_handle.get_webview_window("main") {
            main_window.set_effects(None)?;
        }
    }

    Ok(())
}
