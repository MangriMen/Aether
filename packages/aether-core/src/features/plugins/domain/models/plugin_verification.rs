use serde::{Deserialize, Serialize};

use super::PluginSourceType;

/// Trust-on-first-use record for a plugin's wasm blob.
///
/// Written when the plugin is installed or updated through a remote provider, the only moment
/// the bytes are known to come from the release; every later load refuses a file whose digest
/// moved. Plugins from a local archive have no record: they load, reported as unverified.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginVerification {
    /// Lowercase hex sha256 of the wasm file named by `LoadConfig::Extism.file`.
    pub wasm_sha256: String,
    /// Which provider the bytes were fetched from when trust was established.
    pub source_type: PluginSourceType,
}

impl PluginVerification {
    pub fn new(wasm_sha256: impl Into<String>, source_type: PluginSourceType) -> Self {
        Self {
            wasm_sha256: wasm_sha256.into(),
            source_type,
        }
    }
}
