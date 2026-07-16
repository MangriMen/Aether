use super::CapabilityMetadata;

/// Metadata for a `PackManager` capability, as exposed to the UI.
#[derive(Debug, Clone)]
pub struct PackManagerCapabilityMetadata {
    pub base: CapabilityMetadata,

    /// Whether the manager supports installing packs.
    pub supports_install: bool,

    /// Whether the manager supports updating existing packs.
    pub supports_update: bool,

    /// Whether the manager supports checking for updates.
    pub supports_check_updates: bool,

    /// Optional label for a file/URL input field shown in the "Import" UI.
    pub field_label: Option<String>,

    /// List of supported file extensions for import, e.g. [`"toml"`, `"mrpack"`].
    pub supported_extensions: Vec<String>,
}
