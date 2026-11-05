use super::{AllowedPath, IgnoredAllowedPath};

#[derive(Debug, Default, Clone)]
pub struct PluginSettings {
    pub allowed_hosts: Vec<String>,
    /// Folders the user gave the plugin, on top of the ones the core mounts for every plugin.
    pub allowed_paths: Vec<AllowedPath>,
    /// Saved folders that were not read, each with why. Never mounted or granted, but written
    /// back as they were, so a save does not delete a hand-edited entry.
    pub ignored_paths: Vec<IgnoredAllowedPath>,

    /// API version at which the user force-enabled this plugin.
    /// When the launcher updates to a different API version, this is cleared
    /// and the user must re-confirm.
    pub force_enabled_at_api_version: Option<String>,
}
