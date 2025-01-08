use aether_core::state::{self, Instance, ModLoader};

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
    pub memory: state::MemorySettings,
    pub resolution: state::WindowSize,
    pub credentials: state::Credentials,
}
