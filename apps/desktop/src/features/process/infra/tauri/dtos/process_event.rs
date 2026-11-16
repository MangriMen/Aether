use aether_core::features::events::{ProcessEvent, ProcessEventType};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;
use uuid::Uuid;

use crate::core::HookErrorDto;

#[derive(Serialize, Clone, Debug, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ProcessEventDto {
    /// The number of the event, from one counter for all the events the backend sends: the
    /// frontend drops what it has seen already.
    pub seq: u32,
    pub instance_id: String,
    pub process_id: Uuid,
    pub event: ProcessEventTypeDto,
    pub message: String,
    pub exit_code: Option<i32>,
    pub crashed: bool,
    /// Why the hook could not be run, on `post_exit_hook_failed`; `message` holds the same in
    /// English, for logs.
    pub hook_error: Option<HookErrorDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "snake_case")]
pub enum ProcessEventTypeDto {
    Launched,
    Finished,
    PostExitHookFailed,
}

impl From<ProcessEvent> for ProcessEventDto {
    fn from(value: ProcessEvent) -> Self {
        let hook_error = match &value.event {
            ProcessEventType::PostExitHookFailed(err) => Some(err.into()),
            ProcessEventType::Launched | ProcessEventType::Finished => None,
        };
        Self {
            seq: 0,
            instance_id: value.instance_id.to_string(),
            process_id: value.process_id,
            event: (&value.event).into(),
            message: value.message,
            exit_code: value.exit_code,
            crashed: value.crashed,
            hook_error,
        }
    }
}

impl From<&ProcessEventType> for ProcessEventTypeDto {
    fn from(value: &ProcessEventType) -> Self {
        match value {
            ProcessEventType::Launched => Self::Launched,
            ProcessEventType::Finished => Self::Finished,
            ProcessEventType::PostExitHookFailed(_) => Self::PostExitHookFailed,
        }
    }
}
