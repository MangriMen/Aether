use std::ops::Deref;

use crate::features::minecraft::ModLoader;

use super::{CapabilityMetadata, DownloadInstruction};

/// Metadata for a `ContentSource` capability, as exposed to the UI.
#[derive(Debug, Clone)]
pub struct ContentSourceCapabilityMetadata {
    pub base: CapabilityMetadata,
}

impl Deref for ContentSourceCapabilityMetadata {
    type Target = CapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Version-specific information returned by `ContentSource::get_version_info`.
///
/// Contains the resolved version details and a payload that tells the core
/// how to handle this version — either as a single asset download or as a
/// modpack whose raw manifest bytes are returned inline.
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version_id: String,
    pub version_name: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<ModLoader>,
    pub payload: VersionPayload,
}

/// The payload of a resolved version — either a single asset or a modpack.
#[derive(Debug, Clone)]
pub enum VersionPayload {
    /// A single downloadable asset (mod, resource pack, shader, etc.).
    Asset(Box<DownloadInstruction>),
    /// A modpack whose manifest bytes are returned directly by the plugin.
    /// The core will dispatch it to the appropriate `PackLifecycleHandler`.
    Modpack(Box<ModpackPayload>),
}

/// Payload for a modpack version — contains the raw manifest bytes and format ID.
#[derive(Debug, Clone)]
pub struct ModpackPayload {
    /// Identifies the pack format (e.g. "packwiz", "mrpack").
    pub format_id: String,
    /// Raw manifest bytes (e.g. pack.toml content, index.json from mrpack).
    pub manifest_bytes: Vec<u8>,
}
