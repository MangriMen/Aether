use serde::{Deserialize, Serialize};

use crate::features::settings::{
    DefaultInstanceSettings, Hooks, MemorySettings, WindowSettings, WindowSize,
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DefaultInstanceSettingsV1 {
    pub launch_args: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub memory: MemorySettingsV1,
    pub window: WindowSettingsV1,
    pub hooks: HooksV2,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettingsV1 {
    pub maximum: u32,
}

impl Default for MemorySettingsV1 {
    fn default() -> Self {
        Self { maximum: 2048 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct WindowSizeV1(pub u16, pub u16);

impl Default for WindowSizeV1 {
    fn default() -> Self {
        Self(960, 540)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WindowSettingsV1 {
    pub force_fullscreen: bool,
    pub game_resolution: WindowSizeV1,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct HooksV2 {
    pub pre_launch: String,
    pub wrapper: String,
    pub post_exit: String,
}

impl From<&DefaultInstanceSettings> for DefaultInstanceSettingsV1 {
    fn from(domain: &DefaultInstanceSettings) -> Self {
        Self {
            launch_args: domain.launch_args().to_vec(),
            env_vars: domain.env_vars().to_vec(),
            memory: MemorySettingsV1 {
                maximum: domain.memory().maximum(),
            },
            window: WindowSettingsV1 {
                force_fullscreen: domain.window().force_fullscreen(),
                game_resolution: WindowSizeV1(
                    domain.window().game_resolution().width(),
                    domain.window().game_resolution().height(),
                ),
            },
            hooks: HooksV2 {
                pre_launch: domain.hooks().pre_launch().to_string(),
                wrapper: domain.hooks().wrapper().to_string(),
                post_exit: domain.hooks().post_exit().to_string(),
            },
        }
    }
}

impl From<DefaultInstanceSettingsV1> for DefaultInstanceSettings {
    fn from(dto: DefaultInstanceSettingsV1) -> Self {
        let memory = MemorySettings::new(dto.memory.maximum);
        let resolution =
            WindowSize::new(dto.window.game_resolution.0, dto.window.game_resolution.1);
        let window = WindowSettings::new(dto.window.force_fullscreen, resolution);
        let hooks = Hooks::new(dto.hooks.pre_launch, dto.hooks.wrapper, dto.hooks.post_exit);

        Self::new(dto.launch_args, dto.env_vars, memory, window, hooks)
    }
}

impl From<MemorySettingsV1> for MemorySettings {
    fn from(value: MemorySettingsV1) -> Self {
        Self::new(value.maximum)
    }
}

impl From<MemorySettings> for MemorySettingsV1 {
    fn from(value: MemorySettings) -> Self {
        Self {
            maximum: value.maximum(),
        }
    }
}

impl From<WindowSizeV1> for WindowSize {
    fn from(value: WindowSizeV1) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<WindowSize> for WindowSizeV1 {
    fn from(value: WindowSize) -> Self {
        Self(value.width(), value.height())
    }
}

impl From<WindowSettingsV1> for WindowSettings {
    fn from(value: WindowSettingsV1) -> Self {
        Self::new(value.force_fullscreen, value.game_resolution.into())
    }
}

impl From<WindowSettings> for WindowSettingsV1 {
    fn from(value: WindowSettings) -> Self {
        Self {
            force_fullscreen: value.force_fullscreen(),
            game_resolution: value.game_resolution().into(),
        }
    }
}

impl From<HooksV2> for Hooks {
    fn from(value: HooksV2) -> Self {
        Self::new(value.pre_launch, value.wrapper, value.post_exit)
    }
}

impl From<Hooks> for HooksV2 {
    fn from(value: Hooks) -> Self {
        Self {
            pre_launch: value.pre_launch().to_string(),
            wrapper: value.wrapper().to_string(),
            post_exit: value.post_exit().to_string(),
        }
    }
}
