use tauri::{utils::config::WindowEffectsConfig, window::Effect, AppHandle, Manager};

use crate::state::WindowEffect;

pub fn set_window_effect(app_handle: AppHandle, window_effect: WindowEffect) -> tauri::Result<()> {
    if let Some(main_window) = app_handle.get_webview_window("main") {
        let effect = match window_effect {
            WindowEffect::Off => None,
            WindowEffect::MicaLight => Some(Effect::MicaLight),
            WindowEffect::MicaDark => Some(Effect::MicaDark),
            WindowEffect::Mica => Some(Effect::Mica),
            WindowEffect::Acrylic => Some(Effect::Acrylic),
        };

        main_window.set_effects(None)?;

        if let Some(effect) = effect {
            main_window.set_effects(WindowEffectsConfig {
                effects: vec![effect],
                state: Some(tauri::window::EffectState::FollowsWindowActiveState),
                radius: None,
                color: None,
            })?;
        }
    }

    Ok(())
}
