use aether_core::features::events::WarningEvent;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct WarningEventDto {
    pub message: String,
}

impl From<WarningEvent> for WarningEventDto {
    fn from(value: WarningEvent) -> Self {
        Self {
            message: value.message,
        }
    }
}
