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
    use rstest::rstest;

    // ── AppSettings → AppSettingsDto ──

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

    #[rstest]
    #[case::nothing(ActionOnInstanceLaunch::Nothing, ActionOnInstanceLaunchDto::Nothing)]
    #[case::hide(ActionOnInstanceLaunch::Hide, ActionOnInstanceLaunchDto::Hide)]
    #[case::close(ActionOnInstanceLaunch::Close, ActionOnInstanceLaunchDto::Close)]
    fn action_domain_to_dto(
        #[case] domain: ActionOnInstanceLaunch,
        #[case] expected: ActionOnInstanceLaunchDto,
    ) {
        assert_eq!(ActionOnInstanceLaunchDto::from(domain), expected);
    }

    #[rstest]
    #[case::nothing(ActionOnInstanceLaunchDto::Nothing, ActionOnInstanceLaunch::Nothing)]
    #[case::hide(ActionOnInstanceLaunchDto::Hide, ActionOnInstanceLaunch::Hide)]
    #[case::close(ActionOnInstanceLaunchDto::Close, ActionOnInstanceLaunch::Close)]
    fn action_dto_to_domain(
        #[case] dto: ActionOnInstanceLaunchDto,
        #[case] expected: ActionOnInstanceLaunch,
    ) {
        assert_eq!(ActionOnInstanceLaunch::from(dto), expected);
    }

    // ── WindowEffect conversions ──

    #[rstest]
    #[case::off(WindowEffect::Off, WindowEffectDto::Off)]
    #[case::mica_light(WindowEffect::MicaLight, WindowEffectDto::MicaLight)]
    #[case::mica_dark(WindowEffect::MicaDark, WindowEffectDto::MicaDark)]
    #[case::mica(WindowEffect::Mica, WindowEffectDto::Mica)]
    #[case::acrylic(WindowEffect::Acrylic, WindowEffectDto::Acrylic)]
    fn window_effect_domain_to_dto(
        #[case] domain: WindowEffect,
        #[case] expected: WindowEffectDto,
    ) {
        assert_eq!(WindowEffectDto::from(domain), expected);
    }

    #[rstest]
    #[case::off(WindowEffectDto::Off, WindowEffect::Off)]
    #[case::mica_light(WindowEffectDto::MicaLight, WindowEffect::MicaLight)]
    #[case::mica_dark(WindowEffectDto::MicaDark, WindowEffect::MicaDark)]
    #[case::mica(WindowEffectDto::Mica, WindowEffect::Mica)]
    #[case::acrylic(WindowEffectDto::Acrylic, WindowEffect::Acrylic)]
    fn window_effect_dto_to_domain(#[case] dto: WindowEffectDto, #[case] expected: WindowEffect) {
        assert_eq!(WindowEffect::from(dto), expected);
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
}
