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
    /// The question is over.
    CommandApprovalResolved {
        plugin_id: PluginId,
        request_id: Uuid,
        outcome: CommandApprovalOutcome,
    },
}

/// How a command question ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandApprovalOutcome {
    /// The user said yes or no.
    Answered,
    /// Nobody answered in time: the user may not know the plugin asked.
    TimedOut,
    /// The question was withdrawn: a stop of the plugin was attempted (it may still have been
    /// refused as busy), or the wait was abandoned before it ended.
    Cancelled,
}
