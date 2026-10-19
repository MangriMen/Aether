use aether_core::features::events::CacheEvent;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct CacheEventDto {
    /// The number of the event, from one counter for all the events the backend sends: the
    /// frontend drops what it has seen already.
    pub seq: u32,
    #[serde(flatten)]
    pub kind: CacheEventKindDto,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CacheEventKindDto {
    Changed,
    BusyChanged { busy: bool },
}

impl From<CacheEvent> for CacheEventDto {
    fn from(value: CacheEvent) -> Self {
        Self {
            seq: 0,
            kind: match value {
                CacheEvent::Changed => CacheEventKindDto::Changed,
                CacheEvent::BusyChanged { busy } => CacheEventKindDto::BusyChanged { busy },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_core::features::events::CacheEvent;
    use serde_json::json;

    use super::*;

    #[test]
    fn the_number_sits_beside_the_kind_of_the_event() {
        let dto = CacheEventDto {
            seq: 7,
            ..CacheEventDto::from(CacheEvent::BusyChanged { busy: true })
        };

        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            json!({ "seq": 7, "type": "busyChanged", "busy": true })
        );
    }
}
