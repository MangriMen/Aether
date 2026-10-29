/// A directory in the plugins folder that is not among the plugins, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedPluginDir {
    pub dir_name: String,
    pub reason: SkippedPluginDirReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkippedPluginDirReason {
    /// The directory's name is no plugin id.
    InvalidName,
    /// The plugin in it could not be read; `details` is technical text from the core.
    LoadFailed { details: String },
}
