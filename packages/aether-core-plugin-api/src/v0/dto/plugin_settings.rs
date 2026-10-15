use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettingsDto {
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    #[serde(default)]
    pub allowed_paths: Vec<(String, String)>,
}
