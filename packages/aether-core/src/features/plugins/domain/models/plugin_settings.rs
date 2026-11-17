use super::{AllowedPath, CommandApproval, IgnoredAllowedPath, LocalHost};

#[derive(Debug, Default, Clone)]
pub struct PluginSettings {
    pub allowed_hosts: Vec<String>,
    /// Local hosts the user let the plugin reach at a non-public address (see [`LocalHost`]).
    /// Never edited from the settings form: one is added by confirming the offer an
    /// `addressBlocked` failure makes, and only taken back.
    pub local_hosts: Vec<LocalHost>,
    /// Folders the user gave the plugin, on top of the ones the core mounts for every plugin.
    pub allowed_paths: Vec<AllowedPath>,
    /// Saved folders that were not read, each with why. Never mounted or granted, but written
    /// back as they were, so a save does not delete a hand-edited entry.
    pub ignored_paths: Vec<IgnoredAllowedPath>,

    /// API version at which the user force-enabled this plugin.
    /// When the launcher updates to a different API version, this is cleared
    /// and the user must re-confirm.
    pub force_enabled_at_api_version: Option<String>,

    /// Commands the user approved for good. Never edited from the settings form: an approval is
    /// added by answering the question a plugin's command raises, and only taken back.
    pub command_approvals: Vec<CommandApproval>,
}
