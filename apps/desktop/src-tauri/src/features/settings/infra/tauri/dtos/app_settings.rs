use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::settings::{ActionOnInstanceLaunch, AppSettings, WindowEffect};

#[derive(Clone, Copy, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsDto {
    pub action_on_instance_launch: ActionOnInstanceLaunchDto,
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
