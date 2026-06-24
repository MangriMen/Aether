use super::PathMapping;

#[derive(Debug, Default, Clone)]
pub struct PluginSettings {
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<PathMapping>,

    /// API version at which the user force-enabled this plugin.
    /// When the launcher updates to a different API version, this is cleared
    /// and the user must re-confirm.
    pub force_enabled_at_api_version: Option<String>,
}
