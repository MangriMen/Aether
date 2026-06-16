use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::settings::{ActionOnInstanceLaunch, AppSettings, WindowEffect};

#[derive(Clone, Copy, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsDto {
    pub action_on_instance_launch: ActionOnInstanceLaunchDto,
    pub is_actual_transparent: bool,
    pub transparent: bool,
    pub window_effect: WindowEffectDto,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default, Type)]
#[serde(rename_all = "snake_case")]
pub enum ActionOnInstanceLaunchDto {
    #[default]
    Nothing,
    Hide,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default, Type)]
#[serde(rename_all = "snake_case")]
pub enum WindowEffectDto {
    #[default]
    Off,
    MicaLight,
    MicaDark,
    Mica,
    Acrylic,
}

impl From<AppSettings> for AppSettingsDto {
    fn from(value: AppSettings) -> Self {
        Self {
            action_on_instance_launch: value.action_on_instance_launch.into(),
            is_actual_transparent: value.is_actual_transparent,
            transparent: value.transparent,
            window_effect: value.window_effect.into(),
        }
    }
}

impl From<ActionOnInstanceLaunch> for ActionOnInstanceLaunchDto {
    fn from(value: ActionOnInstanceLaunch) -> Self {
        match value {
            ActionOnInstanceLaunch::Nothing => Self::Nothing,
            ActionOnInstanceLaunch::Hide => Self::Hide,
            ActionOnInstanceLaunch::Close => Self::Close,
        }
    }
}

impl From<ActionOnInstanceLaunchDto> for ActionOnInstanceLaunch {
    fn from(value: ActionOnInstanceLaunchDto) -> Self {
        match value {
            ActionOnInstanceLaunchDto::Nothing => Self::Nothing,
            ActionOnInstanceLaunchDto::Hide => Self::Hide,
            ActionOnInstanceLaunchDto::Close => Self::Close,
        }
    }
}

impl From<WindowEffect> for WindowEffectDto {
    fn from(value: WindowEffect) -> Self {
        match value {
            WindowEffect::Off => Self::Off,
            WindowEffect::MicaLight => Self::MicaLight,
            WindowEffect::MicaDark => Self::MicaDark,
            WindowEffect::Mica => Self::Mica,
            WindowEffect::Acrylic => Self::Acrylic,
        }
    }
}

impl From<WindowEffectDto> for WindowEffect {
    fn from(value: WindowEffectDto) -> Self {
        match value {
            WindowEffectDto::Off => Self::Off,
            WindowEffectDto::MicaLight => Self::MicaLight,
            WindowEffectDto::MicaDark => Self::MicaDark,
            WindowEffectDto::Mica => Self::Mica,
            WindowEffectDto::Acrylic => Self::Acrylic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::settings::AppSettings;

    // ── AppSettings → AppSettingsDto ──

    #[test]
    fn app_settings_to_dto_default() {
        let domain = AppSettings::default();
        let dto: AppSettingsDto = domain.into();
        assert_eq!(
            dto.action_on_instance_launch,
            ActionOnInstanceLaunchDto::Nothing
        );
        assert!(!dto.transparent);
        assert!(!dto.is_actual_transparent);
        assert_eq!(dto.window_effect, WindowEffectDto::Off);
    }

    #[test]
    fn app_settings_to_dto_custom() {
        let domain = AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Close,
            is_actual_transparent: true,
            transparent: true,
            window_effect: WindowEffect::Acrylic,
        };
        let dto: AppSettingsDto = domain.into();
        assert_eq!(
            dto.action_on_instance_launch,
            ActionOnInstanceLaunchDto::Close
        );
        assert!(dto.transparent);
        assert!(dto.is_actual_transparent);
        assert_eq!(dto.window_effect, WindowEffectDto::Acrylic);
    }

    // ── ActionOnInstanceLaunch conversions ──

    #[test]
    fn action_domain_to_dto_all_variants() {
        assert_eq!(
            ActionOnInstanceLaunchDto::from(ActionOnInstanceLaunch::Nothing),
            ActionOnInstanceLaunchDto::Nothing
        );
        assert_eq!(
            ActionOnInstanceLaunchDto::from(ActionOnInstanceLaunch::Hide),
            ActionOnInstanceLaunchDto::Hide
        );
        assert_eq!(
            ActionOnInstanceLaunchDto::from(ActionOnInstanceLaunch::Close),
            ActionOnInstanceLaunchDto::Close
        );
    }

    #[test]
    fn action_dto_to_domain_all_variants() {
        assert_eq!(
            ActionOnInstanceLaunch::from(ActionOnInstanceLaunchDto::Nothing),
            ActionOnInstanceLaunch::Nothing
        );
        assert_eq!(
            ActionOnInstanceLaunch::from(ActionOnInstanceLaunchDto::Hide),
            ActionOnInstanceLaunch::Hide
        );
        assert_eq!(
            ActionOnInstanceLaunch::from(ActionOnInstanceLaunchDto::Close),
            ActionOnInstanceLaunch::Close
        );
    }

    #[test]
    fn action_dto_roundtrip_all_variants() {
        let variants = [
            ActionOnInstanceLaunch::Nothing,
            ActionOnInstanceLaunch::Hide,
            ActionOnInstanceLaunch::Close,
        ];
        for variant in variants {
            let dto: ActionOnInstanceLaunchDto = variant.into();
            let back: ActionOnInstanceLaunch = dto.into();
            assert_eq!(variant, back);
        }
    }

    // ── WindowEffect conversions ──

    #[test]
    fn window_effect_domain_to_dto_all_variants() {
        assert_eq!(
            WindowEffectDto::from(WindowEffect::Off),
            WindowEffectDto::Off
        );
        assert_eq!(
            WindowEffectDto::from(WindowEffect::MicaLight),
            WindowEffectDto::MicaLight
        );
        assert_eq!(
            WindowEffectDto::from(WindowEffect::MicaDark),
            WindowEffectDto::MicaDark
        );
        assert_eq!(
            WindowEffectDto::from(WindowEffect::Mica),
            WindowEffectDto::Mica
        );
        assert_eq!(
            WindowEffectDto::from(WindowEffect::Acrylic),
            WindowEffectDto::Acrylic
        );
    }

    #[test]
    fn window_effect_dto_to_domain_all_variants() {
        assert_eq!(WindowEffect::from(WindowEffectDto::Off), WindowEffect::Off);
        assert_eq!(
            WindowEffect::from(WindowEffectDto::MicaLight),
            WindowEffect::MicaLight
        );
        assert_eq!(
            WindowEffect::from(WindowEffectDto::MicaDark),
            WindowEffect::MicaDark
        );
        assert_eq!(
            WindowEffect::from(WindowEffectDto::Mica),
            WindowEffect::Mica
        );
        assert_eq!(
            WindowEffect::from(WindowEffectDto::Acrylic),
            WindowEffect::Acrylic
        );
    }

    #[test]
    fn window_effect_dto_roundtrip_all_variants() {
        let variants = [
            WindowEffect::Off,
            WindowEffect::MicaLight,
            WindowEffect::MicaDark,
            WindowEffect::Mica,
            WindowEffect::Acrylic,
        ];
        for variant in variants {
            let dto: WindowEffectDto = variant.into();
            let back: WindowEffect = dto.into();
            assert_eq!(variant, back);
        }
    }

    // ── JSON serialization ──

    #[test]
    fn app_settings_dto_serialize_camel_case() {
        let dto = AppSettingsDto {
            action_on_instance_launch: ActionOnInstanceLaunchDto::Hide,
            is_actual_transparent: false,
            transparent: true,
            window_effect: WindowEffectDto::Mica,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("windowEffect"), "Expected camelCase: {json}");
        assert!(
            json.contains("actionOnInstanceLaunch"),
            "Expected camelCase: {json}"
        );
        assert!(
            json.contains("isActualTransparent"),
            "Expected camelCase: {json}"
        );
        assert!(!json.contains("window_effect"), "Got snake_case: {json}");
    }

    #[test]
    fn app_settings_dto_deserialize_camel_case() {
        let json = r#"{
            "actionOnInstanceLaunch": "close",
            "isActualTransparent": true,
            "transparent": true,
            "windowEffect": "mica_dark"
        }"#;
        let dto: AppSettingsDto = serde_json::from_str(json).unwrap();
        assert_eq!(
            dto.action_on_instance_launch,
            ActionOnInstanceLaunchDto::Close
        );
        assert!(dto.is_actual_transparent);
        assert!(dto.transparent);
        assert_eq!(dto.window_effect, WindowEffectDto::MicaDark);
    }

    #[test]
    fn app_settings_dto_default_is_off() {
        let dto = AppSettingsDto::default();
        assert_eq!(
            dto.action_on_instance_launch,
            ActionOnInstanceLaunchDto::Nothing
        );
        assert_eq!(dto.window_effect, WindowEffectDto::Off);
    }
}
