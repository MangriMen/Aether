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

pub struct ArchiveMetadata {
    pub manifest: PluginManifest,
    /// `capabilities.json` as written, already validated; `None` when absent.
    pub capabilities: Option<String>,
}
