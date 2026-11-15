use aether_core::features::events::{CommandApprovalOutcome, PluginEvent};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct PluginEventDto {
    /// The number of the event, from one counter for all the events the backend sends: the
    /// frontend drops what it has seen already.
    pub seq: u32,
    #[serde(flatten)]
    pub kind: PluginEventKindDto,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PluginEventKindDto {
    Edit {
        plugin_id: String,
    },
    Sync,
    /// A command of the plugin waits for the user. The arguments are not here: the frontend asks
    /// for the question.
    CommandApprovalRequested {
        plugin_id: String,
        request_id: String,
    },
    CommandApprovalResolved {
        plugin_id: String,
        request_id: String,
        outcome: CommandApprovalOutcomeDto,
    },
}

/// How a command question ended. The UI tells the user about a timeout only: an answer is
/// theirs, and a cancel follows an attempt to stop the plugin, which they started.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub enum CommandApprovalOutcomeDto {
    Answered,
    TimedOut,
    Cancelled,
}

impl From<CommandApprovalOutcome> for CommandApprovalOutcomeDto {
    fn from(value: CommandApprovalOutcome) -> Self {
        match value {
            CommandApprovalOutcome::Answered => Self::Answered,
            CommandApprovalOutcome::TimedOut => Self::TimedOut,
            CommandApprovalOutcome::Cancelled => Self::Cancelled,
        }
    }
}

impl From<PluginEvent> for PluginEventDto {
    fn from(value: PluginEvent) -> Self {
        Self {
            seq: 0,
            kind: match value {
                PluginEvent::Edit { plugin_id } => PluginEventKindDto::Edit {
                    plugin_id: plugin_id.into(),
                },
                PluginEvent::Sync => PluginEventKindDto::Sync,
                PluginEvent::CommandApprovalRequested {
                    plugin_id,
                    request_id,
                } => PluginEventKindDto::CommandApprovalRequested {
                    plugin_id: plugin_id.into(),
                    request_id: request_id.to_string(),
                },
                PluginEvent::CommandApprovalResolved {
                    plugin_id,
                    request_id,
                    outcome,
                } => PluginEventKindDto::CommandApprovalResolved {
                    plugin_id: plugin_id.into(),
                    request_id: request_id.to_string(),
                    outcome: outcome.into(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_core::features::{events::PluginEvent, instance::PluginId};
    use serde_json::json;

    use super::*;

    #[test]
    fn the_number_sits_beside_the_kind_of_the_event() {
        let dto = PluginEventDto {
            seq: 3,
            ..PluginEventDto::from(PluginEvent::Edit {
                plugin_id: PluginId::from_static("a"),
            })
        };

        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            json!({ "seq": 3, "type": "edit", "plugin_id": "a" })
        );
    }

    #[test]
    fn should_carry_how_it_ended_and_nothing_about_the_command_when_a_question_resolves() {
        let dto = PluginEventDto {
            seq: 1,
            ..PluginEventDto::from(PluginEvent::CommandApprovalResolved {
                plugin_id: PluginId::from_static("a"),
                request_id: uuid::Uuid::nil(),
                outcome: CommandApprovalOutcome::TimedOut,
            })
        };

        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            json!({
                "seq": 1,
                "type": "commandApprovalResolved",
                "plugin_id": "a",
                "request_id": "00000000-0000-0000-0000-000000000000",
                "outcome": "timedOut"
            })
        );
    }
}
