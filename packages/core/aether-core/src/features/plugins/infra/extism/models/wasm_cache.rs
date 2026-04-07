use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WasmCacheConfig {
    pub cache: WasmCache,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct WasmCache {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: String,
    #[serde(default = "default_files_total_size_soft_limit")]
    pub files_total_size_soft_limit: String,
    pub directory: PathBuf,
}

fn default_enabled() -> bool {
    true
}

fn default_cleanup_interval() -> String {
    "30m".to_owned()
}

fn default_files_total_size_soft_limit() -> String {
    "1Gi".to_owned()
}
