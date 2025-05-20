use aether_core::features::{
    auth::Credentials,
    instance::Instance,
    settings::{MemorySettings, WindowSize},
};

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
pub struct InstanceImportDto {
    pub pack_type: String,
    pub path: String,
}
