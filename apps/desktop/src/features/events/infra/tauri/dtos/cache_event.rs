use aether_core::features::events::CacheEvent;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CacheEventDto {
    Changed,
    BusyChanged { busy: bool },
}

impl From<CacheEvent> for CacheEventDto {
    fn from(value: CacheEvent) -> Self {
        match value {
            CacheEvent::Changed => Self::Changed,
            CacheEvent::BusyChanged { busy } => Self::BusyChanged { busy },
        }
    }
}
