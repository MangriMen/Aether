use aether_core::features::events::{ProcessEvent, ProcessEventType};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ProcessEventDto {
    pub instance_id: String,
    pub process_id: Uuid,
    pub event: ProcessEventTypeDto,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "snake_case")]
pub enum ProcessEventTypeDto {
    Launched,
    Finished,
}

impl From<ProcessEvent> for ProcessEventDto {
    fn from(value: ProcessEvent) -> Self {
        Self {
            instance_id: value.instance_id,
            process_id: value.process_id,
            event: value.event.into(),
            message: value.message,
        }
    }
}

impl From<ProcessEventType> for ProcessEventTypeDto {
    fn from(value: ProcessEventType) -> Self {
        match value {
            ProcessEventType::Launched => Self::Launched,
            ProcessEventType::Finished => Self::Finished,
        }
    }
}
