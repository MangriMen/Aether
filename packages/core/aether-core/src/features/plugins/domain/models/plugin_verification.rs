use serde::{Deserialize, Serialize};

use super::PluginSourceType;

/// Trust-on-first-use record for a plugin's wasm blob.
///
/// Written once, when the plugin is installed or updated through a remote
/// provider — the only moment the bytes are known to come from the release the
/// user picked. Every later load re-hashes the file on disk and refuses to run
/// it if the digest moved (see `OPEN_QUESTIONS.md`, Q4).
///
/// Plugins installed from a local archive get no record at all: they load, but
/// are reported as unverified.
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
