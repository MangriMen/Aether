use aether_core::features::events::PluginEvent;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct PluginEventDto {
    /// The place of the event in the order the backend sent all events of its type: the
    /// frontend drops what it has seen already.
    pub seq: u32,
    #[serde(flatten)]
    pub kind: PluginEventKindDto,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PluginEventKindDto {
    Add { plugin_id: String },
    Edit { plugin_id: String },
    Remove { plugin_id: String },
    Sync,
}

impl From<PluginEvent> for PluginEventDto {
    fn from(value: PluginEvent) -> Self {
        Self {
            seq: 0,
            kind: match value {
                PluginEvent::Add { plugin_id } => PluginEventKindDto::Add { plugin_id },
                PluginEvent::Edit { plugin_id } => PluginEventKindDto::Edit { plugin_id },
                PluginEvent::Remove { plugin_id } => PluginEventKindDto::Remove { plugin_id },
                PluginEvent::Sync => PluginEventKindDto::Sync,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_core::features::events::PluginEvent;
    use serde_json::json;

    use super::*;

    #[test]
    fn the_number_sits_beside_the_kind_of_the_event() {
        let dto = PluginEventDto {
            seq: 3,
            ..PluginEventDto::from(PluginEvent::Add {
                plugin_id: "a".to_owned(),
            })
        };

        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            json!({ "seq": 3, "type": "add", "plugin_id": "a" })
        );
    }
}
