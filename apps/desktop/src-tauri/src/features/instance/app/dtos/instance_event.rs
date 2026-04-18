use aether_core::features::events::{InstanceEvent, InstanceEventType};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct InstanceEventDto {
    pub event: InstanceEventTypeDto,
    pub instance_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "snake_case")]
pub enum InstanceEventTypeDto {
    Created,
    Synced,
    Edited,
    Removed,
}

impl From<InstanceEvent> for InstanceEventDto {
    fn from(value: InstanceEvent) -> Self {
        Self {
            event: value.event.into(),
            instance_id: value.instance_id,
        }
    }
}

impl From<InstanceEventType> for InstanceEventTypeDto {
    fn from(value: InstanceEventType) -> Self {
        match value {
            InstanceEventType::Created => Self::Created,
            InstanceEventType::Synced => Self::Synced,
            InstanceEventType::Edited => Self::Edited,
            InstanceEventType::Removed => Self::Removed,
        }
    }
}
