use aether_core::features::events::WarningEvent;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct WarningEventDto {
    /// The place of the event in the order the backend sent all events of its type: the
    /// frontend drops what it has seen already.
    pub seq: u32,
    pub message: String,
}

impl From<WarningEvent> for WarningEventDto {
    fn from(value: WarningEvent) -> Self {
        Self {
            seq: 0,
            message: value.message,
        }
    }
}
