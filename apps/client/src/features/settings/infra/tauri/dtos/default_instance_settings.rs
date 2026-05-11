use aether_core::features::settings::{
    DefaultInstanceSettings, Hooks, MemorySettings, WindowSettings, WindowSize,
};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct DefaultInstanceSettingsDto {
    launch_args: Vec<String>,
    env_vars: Vec<(String, String)>,

    memory: MemorySettingsDto,
    window: WindowSettingsDto,

    hooks: HooksDto,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Type)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettingsDto {
    pub maximum: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Type)]
pub struct WindowSizeDto(pub u16, pub u16);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Type)]
#[serde(rename_all = "camelCase")]
pub struct WindowSettingsDto {
    pub force_fullscreen: bool,
    pub game_resolution: WindowSizeDto,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
#[serde(rename_all = "camelCase")]
pub struct HooksDto {
    pre_launch: String,
    wrapper: String,
    post_exit: String,
}

impl From<DefaultInstanceSettings> for DefaultInstanceSettingsDto {
    fn from(value: DefaultInstanceSettings) -> Self {
        Self {
            launch_args: value.launch_args().to_vec(),
            env_vars: value.env_vars().to_vec(),
            memory: (*value.memory()).into(),
            window: value.window().clone().into(),
            hooks: value.hooks().clone().into(),
        }
    }
}

impl From<MemorySettings> for MemorySettingsDto {
    fn from(value: MemorySettings) -> Self {
        Self {
            maximum: value.maximum,
        }
    }
}

impl From<MemorySettingsDto> for MemorySettings {
    fn from(value: MemorySettingsDto) -> Self {
        Self {
            maximum: value.maximum,
        }
    }
}

impl From<WindowSize> for WindowSizeDto {
    fn from(value: WindowSize) -> Self {
        Self(value.0, value.1)
    }
}

impl From<WindowSizeDto> for WindowSize {
    fn from(value: WindowSizeDto) -> Self {
        Self(value.0, value.1)
    }
}

impl From<WindowSettingsDto> for WindowSettings {
    fn from(value: WindowSettingsDto) -> Self {
        WindowSettings::new(value.force_fullscreen, value.game_resolution.into())
    }
}

impl From<WindowSettings> for WindowSettingsDto {
    fn from(value: WindowSettings) -> Self {
        Self {
            force_fullscreen: value.force_fullscreen(),
            game_resolution: value.game_resolution().into(),
        }
    }
}

impl From<Hooks> for HooksDto {
    fn from(value: Hooks) -> Self {
        Self {
            pre_launch: value.pre_launch().to_owned(),
            wrapper: value.wrapper().to_owned(),
            post_exit: value.post_exit().to_owned(),
        }
    }
}

impl From<HooksDto> for Hooks {
    fn from(value: HooksDto) -> Self {
        Self::new(value.pre_launch, value.wrapper, value.post_exit)
    }
}
