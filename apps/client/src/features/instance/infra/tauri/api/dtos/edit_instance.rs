use aether_core::features::instance::app::EditInstance;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::settings::infra::tauri::dtos::{
    EditHooksDto, MemorySettingsDto, WindowSizeDto,
};

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditInstanceDto {
    #[specta(optional)]
    pub name: Option<String>,

    #[specta(optional, type = Option<String>)]
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub java_path: Option<Option<String>>,

    #[specta(optional, type = Option<Vec<String>>)]
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub launch_args: Option<Option<Vec<String>>>,

    #[specta(optional, type = Option<Vec<(String, String)>>)]
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub env_vars: Option<Option<Vec<(String, String)>>>,

    #[specta(optional, type = Option<MemorySettingsDto>)]
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub memory: Option<Option<MemorySettingsDto>>,

    #[specta(optional, type = Option<WindowSizeDto>)]
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub game_resolution: Option<Option<WindowSizeDto>>,

    #[specta(optional)]
    pub hooks: Option<EditHooksDto>,
}

impl From<EditInstanceDto> for EditInstance {
    fn from(value: EditInstanceDto) -> Self {
        Self {
            name: value.name,
            java_path: value.java_path,
            launch_args: value.launch_args,
            env_vars: value.env_vars,
            memory: value.memory.map(|o| o.map(Into::into)),
            game_resolution: value.game_resolution.map(|o| o.map(Into::into)),
            hooks: value.hooks.map(Into::into),
        }
    }
}
