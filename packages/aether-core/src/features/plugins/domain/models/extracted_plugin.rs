use tempfile::TempDir;

use crate::features::plugins::PluginManifest;

pub struct ExtractedPlugin {
    pub plugin_id: String,
    pub manifest: PluginManifest,
    pub content: PluginContent,
}

pub enum PluginContent {
    Filesystem { temp_dir: TempDir },
}
