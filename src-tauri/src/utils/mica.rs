use tauri::{utils::config::WindowEffectsConfig, window::Effect, AppHandle, Manager};

use crate::state::MicaMode;

pub fn set_mica(app_handle: AppHandle, mode: MicaMode) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    {
        if let Some(main_window) = app_handle.get_webview_window("main") {
            let effect = match mode {
                MicaMode::Off => None,
                MicaMode::Light => Some(Effect::MicaLight),
                MicaMode::Dark => Some(Effect::MicaDark),
                MicaMode::System => Some(Effect::Mica),
            };

            if let Some(effect) = effect {
                main_window.set_effects(WindowEffectsConfig {
                    effects: vec![effect],
                    state: Some(tauri::window::EffectState::FollowsWindowActiveState),
                    radius: None,
                    color: None,
                })?;
            } else {
                main_window.set_effects(None)?;
            }
        }
    }

    Ok(())
}
