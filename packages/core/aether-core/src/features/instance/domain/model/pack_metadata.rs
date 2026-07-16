use crate::features::minecraft::ModLoader;

/// Metadata extracted from a pack source before installation.
///
/// Returned by `PackManager::resolve_pack_metadata` so the core can create
/// the instance with correct game version, loader, and name — rather than
/// requiring the frontend to guess these values.
#[derive(Debug, Clone)]
pub struct PackMetadata {
    /// Display name for the instance (from the pack).
    pub name: String,
    /// Minecraft version required by the pack.
    pub game_version: String,
    /// Mod loader the pack uses (Vanilla / Fabric / Forge / Quilt / `NeoForge`).
    pub mod_loader: ModLoader,
    /// Optional specific loader version.
    pub loader_version: Option<String>,
    /// Pack identification for the `pack_info` stored on the instance.
    pub pack_info: Option<super::PackInfo>,
}
