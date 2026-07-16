use serde::{Deserialize, Serialize};

/// Describes where a pack comes from — passed to the plugin's `resolve_pack_metadata` handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PackSourceDto {
    /// A local pack file path (e.g., `pack.toml`).
    LocalFile(String),
    /// A remote pack URL.
    RemoteUrl(String),
}
