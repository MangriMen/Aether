use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DefaultInstanceSettingsDto {
    pub launch_args: Vec<String>,
    pub env_vars: Vec<(String, String)>,

    pub memory: MemorySettingsDto,
    pub window: WindowSettingsDto,

    pub hooks: HooksDto,
}

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettingsDto {
    pub maximum: u32,
}

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone, Copy)]
pub struct WindowSizeDto(pub u16, pub u16);

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct WindowSettingsDto {
    pub force_fullscreen: bool,
    pub game_resolution: WindowSizeDto,
}

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct HooksDto {
    pub pre_launch: String,
    pub wrapper: String,
    pub post_exit: String,
}
