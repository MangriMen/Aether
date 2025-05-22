use aether_core::features::{
    auth::Credentials,
    instance::Instance,
    settings::{MemorySettings, WindowSize},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceLaunchDto {
    pub instance: Instance,
    pub env_args: Vec<(String, String)>,
    pub java_args: Vec<String>,
    pub memory: MemorySettings,
    pub resolution: WindowSize,
    pub credentials: Credentials,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceImportDto {
    pub pack_type: String,
    pub path: String,
}
