use aether_core::features::settings::{EditDefaultInstanceSettings, EditHooks, SettingsError};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::{MemorySettingsDto, WindowSettingsDto};

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditDefaultInstanceSettingsDto {
    #[specta(optional)]
    pub launch_args: Option<Vec<String>>,

    #[specta(optional)]
    pub env_vars: Option<Vec<(String, String)>>,

    #[specta(optional)]
    pub memory: Option<MemorySettingsDto>,

    #[specta(optional)]
    pub window: Option<WindowSettingsDto>,

    #[specta(optional)]
    pub hooks: Option<EditHooksDto>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditHooksDto {
    #[specta(optional)]
    pub pre_launch: Option<String>,

    #[specta(optional)]
    pub wrapper: Option<String>,

    #[specta(optional)]
    pub post_exit: Option<String>,
}

impl TryFrom<EditDefaultInstanceSettingsDto> for EditDefaultInstanceSettings {
    type Error = SettingsError;

    fn try_from(value: EditDefaultInstanceSettingsDto) -> Result<Self, Self::Error> {
        Ok(Self {
            launch_args: value.launch_args,
            env_vars: value.env_vars,
            memory: value.memory.map(TryInto::try_into).transpose()?,
            window: value.window.map(Into::into),
            hooks: value.hooks.map(Into::into),
        })
    }
}

impl From<EditHooksDto> for EditHooks {
    fn from(value: EditHooksDto) -> Self {
        Self {
            pre_launch: value.pre_launch.clone(),
            wrapper: value.wrapper.clone(),
            post_exit: value.post_exit.clone(),
        }
    }
}
