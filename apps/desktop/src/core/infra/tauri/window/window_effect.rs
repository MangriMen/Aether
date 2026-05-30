#![allow(clippy::needless_pass_by_value)]
use tauri::{
    WebviewWindow,
    utils::config::WindowEffectsConfig,
    window::{Effect, EffectState},
};

use crate::features::settings::WindowEffect;

/// Applies the specified effect with default configuration
pub fn apply_effect_to_window<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    effect: Option<Effect>,
) -> tauri::Result<()> {
    log::debug!("Applying window effect: {effect:?}");

    // Using map to transform Option<Effect> into Option<WindowEffectsConfig>
    let config = effect.map(|e| WindowEffectsConfig {
        effects: vec![e],
        state: Some(EffectState::FollowsWindowActiveState),
        radius: None,
        color: None,
    });

    window.set_effects(config)
}

impl From<WindowEffect> for Option<Effect> {
    fn from(value: WindowEffect) -> Self {
        match value {
            WindowEffect::Off => None,
            WindowEffect::MicaLight => Some(Effect::MicaLight),
            WindowEffect::MicaDark => Some(Effect::MicaDark),
            WindowEffect::Mica => Some(Effect::Mica),
            WindowEffect::Acrylic => Some(Effect::Acrylic),
        }
    }
}
