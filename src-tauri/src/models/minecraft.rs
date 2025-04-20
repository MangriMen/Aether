use aether_core::features::{
    auth::Credentials,
    instance::Instance,
    minecraft::ModLoader,
    settings::{MemorySettings, WindowSize},
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstanceCreateDto {
    pub name: String,
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub loader_version: Option<String>,
    pub icon_path: Option<String>,
    pub skip_install_profile: Option<bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceLaunchDto {
    pub instance: Instance,
    pub env_args: Vec<(String, String)>,
    pub java_args: Vec<String>,
    pub memory: MemorySettings,
    pub resolution: WindowSize,
    pub credentials: Credentials,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceEditDto {
    pub name: Option<String>,
    pub java_path: Option<String>,
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub game_resolution: Option<WindowSize>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceImportDto {
    pub pack_type: String,
    pub path: String,
}
