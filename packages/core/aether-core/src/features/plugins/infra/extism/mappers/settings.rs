use aether_core_plugin_api::v0::{
    DefaultInstanceSettingsDto, HooksDto, MemorySettingsDto, WindowSettingsDto, WindowSizeDto,
};

use crate::features::settings::{
    DefaultInstanceSettings, Hooks, MemorySettings, WindowSettings, WindowSize,
};

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
        Self(value.width(), value.height())
    }
}

impl From<WindowSizeDto> for WindowSize {
    fn from(value: WindowSizeDto) -> Self {
        Self::new(value.0, value.1)
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
