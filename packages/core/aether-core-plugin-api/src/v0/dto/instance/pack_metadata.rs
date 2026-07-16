use serde::{Deserialize, Serialize};

use crate::v0::{ModLoaderDto, PackInfoDto};

/// Metadata extracted from a pack source before instance creation.
/// Returned by the plugin's `resolve_pack_metadata` handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMetadataDto {
    /// Display name for the instance (from the pack).
    pub name: String,
    /// Minecraft version required by the pack.
    pub game_version: String,
    /// Mod loader the pack uses.
    pub mod_loader: ModLoaderDto,
    /// Optional specific loader version.
    pub loader_version: Option<String>,
    /// Pack identification information.
    pub pack_info: Option<PackInfoDto>,
}
