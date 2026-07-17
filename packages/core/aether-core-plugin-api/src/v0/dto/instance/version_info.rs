use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Identifier for a specific content version — used as input to `get_version_info`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ContentVersionIdentifierDto {
    pub content_id: String,
    pub version_id: String,
}

/// Version information returned by `get_version_info`.
///
/// Contains the resolved version details and a payload that tells the core
/// how to handle this version — either as a single asset download or as a
/// modpack whose raw manifest bytes are returned inline.
///
/// Payload types:
/// - `"asset"`: the plugin expects the host to download a single file
/// - `"modpack"`: the plugin returns raw manifest bytes for the host to process
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct VersionInfoDto {
    pub version_id: String,
    pub version_name: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,

    /// "asset" or "modpack"
    pub payload_type: String,

    // ── Asset payload fields ──
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    // ── Modpack payload fields ──
    /// Format identifier (e.g. "packwiz", "mrpack").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format_id: Option<String>,
    /// Raw manifest bytes (serialized as Msgpack binary).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_bytes: Option<Vec<u8>>,
}