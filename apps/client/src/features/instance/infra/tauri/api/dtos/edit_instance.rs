use aether_core::features::instance::EditInstance;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::settings::{
    dtos::WindowSettingsDto,
    infra::tauri::dtos::{EditHooksDto, MemorySettingsDto},
};

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditInstanceDto {
    #[specta(optional)]
    pub name: Option<String>,

    // Java Path
    #[specta(optional)]
    pub override_java_path: Option<bool>,
    #[specta(optional)]
    pub java_path: Option<String>,

    // Launch Args
    #[specta(optional)]
    pub override_launch_args: Option<bool>,
    #[specta(optional)]
    pub launch_args: Option<Vec<String>>,

    // Env Vars
    #[specta(optional)]
    pub override_env_vars: Option<bool>,
    #[specta(optional)]
    pub env_vars: Option<Vec<(String, String)>>,

    // Memory
    #[specta(optional)]
    pub override_memory: Option<bool>,
    #[specta(optional)]
    pub memory: Option<MemorySettingsDto>,

    // Window
    #[specta(optional)]
    pub override_window_settings: Option<bool>,
    #[specta(optional)]
    pub window: Option<WindowSettingsDto>,

    // Hooks
    #[specta(optional)]
    pub override_hooks: Option<bool>,
    #[specta(optional)]
    pub hooks: Option<EditHooksDto>,
}

impl From<EditInstanceDto> for EditInstance {
    fn from(value: EditInstanceDto) -> Self {
        Self {
            name: value.name,

            override_java_path: value.override_java_path,
            java_path: value.java_path,

            override_launch_args: value.override_launch_args,
            launch_args: value.launch_args,

            override_env_vars: value.override_env_vars,
            env_vars: value.env_vars,

            override_memory: value.override_memory,
            memory: value.memory.map(Into::into),

            override_window_settings: value.override_window_settings,
            window: value.window.map(Into::into),

            override_hooks: value.override_hooks,
            hooks: value.hooks.map(Into::into),
        }
    }
}
