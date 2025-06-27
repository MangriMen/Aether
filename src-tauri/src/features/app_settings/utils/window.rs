use tauri::{
    utils::config::WindowEffectsConfig,
    window::{Effect, EffectState},
    AppHandle, Manager,
};

use crate::features::app_settings::WindowEffect;

/// Applies the specified window effect to the main application window
pub fn set_window_effect(app_handle: AppHandle, window_effect: WindowEffect) -> tauri::Result<()> {
    let Some(main_window) = app_handle.get_webview_window("main") else {
        return Ok(());
    };

    // Clear any existing effects first
    main_window.set_effects(None)?;

    if let Some(effect) = map_to_tauri_effect(window_effect) {
        apply_window_effect(&main_window, effect)?;
    }

    Ok(())
}

/// Maps our application's WindowEffect enum to Tauri's Effect
fn map_to_tauri_effect(window_effect: WindowEffect) -> Option<Effect> {
    match window_effect {
        WindowEffect::Off => None,
        WindowEffect::MicaLight => Some(Effect::MicaLight),
        WindowEffect::MicaDark => Some(Effect::MicaDark),
        WindowEffect::Mica => Some(Effect::Mica),
        WindowEffect::Acrylic => Some(Effect::Acrylic),
    }
}

/// Applies the specified effect to the window with default configuration
fn apply_window_effect(window: &tauri::WebviewWindow, effect: Effect) -> tauri::Result<()> {
    let effects_config = WindowEffectsConfig {
        effects: vec![effect],
        state: Some(EffectState::FollowsWindowActiveState),
        radius: None,
        color: None,
    };

    window.set_effects(effects_config)
}
