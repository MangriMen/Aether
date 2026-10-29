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
