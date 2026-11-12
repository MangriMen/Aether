use tempfile::TempDir;

use crate::features::{instance::PluginId, plugins::PluginManifest};

pub struct ExtractedPlugin {
    pub plugin_id: PluginId,
    pub manifest: PluginManifest,
    pub content: PluginContent,
}

pub enum PluginContent {
    Filesystem { temp_dir: TempDir },
}

/// What an archive says about the plugin, read without unpacking it.
pub struct ArchiveMetadata {
    pub manifest: PluginManifest,
    /// `capabilities.json` as written, checked as loading it would; none when there is none.
    pub capabilities: Option<String>,
}
