use crate::shared::capability::PluginId;

#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// A plugin's state, settings or files changed.
    Edit { plugin_id: PluginId },
    /// The set of plugins changed; the listener reads it again.
    Sync,
}
