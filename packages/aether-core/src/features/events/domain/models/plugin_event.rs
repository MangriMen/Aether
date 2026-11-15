use uuid::Uuid;

use crate::shared::capability::PluginId;

#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// A plugin's state, settings or files changed.
    Edit { plugin_id: PluginId },
    /// The set of plugins changed; the listener reads it again.
    Sync,
    /// A plugin's command waits for the user's answer. Carries no arguments: they can hold a
    /// token, and the UI asks for the request.
    CommandApprovalRequested {
        plugin_id: PluginId,
        request_id: Uuid,
    },
    /// The question is over (answered, timed out or cancelled).
    CommandApprovalResolved {
        plugin_id: PluginId,
        request_id: Uuid,
    },
}
