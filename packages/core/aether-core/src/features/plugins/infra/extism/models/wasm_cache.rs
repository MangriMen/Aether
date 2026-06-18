use std::path::PathBuf;

use serde::Serialize;

/// Wasm cache config written to disk for extism/wasmtime.
/// Note: wasmtime's `CacheConfig` has `#[serde(deny_unknown_fields)]` —
/// only include fields that exist in its struct.
#[derive(Serialize)]
pub struct WasmCacheConfig {
    pub cache: WasmCache,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WasmCache {
    pub cleanup_interval: String,
    pub files_total_size_soft_limit: String,
    pub directory: PathBuf,
}

pub fn get_default_cache_config(cache_dir: PathBuf) -> WasmCacheConfig {
    WasmCacheConfig {
        cache: WasmCache {
            cleanup_interval: "30m".to_owned(),
            files_total_size_soft_limit: "1Gi".to_owned(),
            directory: cache_dir,
        },
    }
}
