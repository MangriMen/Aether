use std::sync::Arc;

use super::PluginError;

/// A directory in the plugins folder that is not among the plugins, and why.
#[derive(Debug, Clone)]
pub struct SkippedPluginDir {
    pub dir_name: String,
    pub reason: SkippedPluginDirReason,
}

#[derive(Debug, Clone)]
pub enum SkippedPluginDirReason {
    /// The directory's name is no plugin id.
    InvalidName,
    /// The plugin in it could not be read. Shared, because the error is not `Clone` and the
    /// listing is handed out more than once.
    LoadFailed { error: Arc<PluginError> },
}
