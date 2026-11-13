use aether_core::features::events::WarningEvent;
use serde::Serialize;
use specta::Type;
use tauri_specta::Event;

use crate::core::CauseDto;

#[derive(Debug, Serialize, Clone, Type, Event)]
pub struct WarningEventDto {
    /// The number of the event, from one counter for all the events the backend sends: the
    /// frontend drops what it has seen already.
    pub seq: u32,
    pub warning: WarningDto,
}

/// What the user is warned of: a code the frontend translates, with its arguments.
#[derive(Debug, Serialize, Clone, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarningDto {
    InstanceIconNotImported { cause: CauseDto },
}

impl From<WarningEvent> for WarningEventDto {
    fn from(value: WarningEvent) -> Self {
        Self {
            seq: 0,
            warning: value.into(),
        }
    }
}

impl From<WarningEvent> for WarningDto {
    fn from(value: WarningEvent) -> Self {
        match value {
            WarningEvent::InstanceIconNotImported { cause } => Self::InstanceIconNotImported {
                cause: cause.as_ref().into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use aether_core::shared::cache::AssetError;
    use serde_json::json;

    use super::*;

    fn to_json(event: WarningEvent) -> serde_json::Value {
        serde_json::to_value(WarningEventDto::from(event)).unwrap()
    }

    #[test]
    fn should_serialize_an_icon_not_imported_with_the_category_of_its_cause() {
        let event = WarningEvent::InstanceIconNotImported {
            cause: Arc::new(AssetError::ReadSource(Arc::new(std::io::Error::from(
                std::io::ErrorKind::PermissionDenied,
            )))),
        };

        assert_eq!(
            to_json(event),
            json!({"seq": 0, "warning": {
                "code": "INSTANCE_ICON_NOT_IMPORTED",
                "payload": {"cause": {"type": "io", "kind": "permissionDenied"}},
            }})
        );
    }

    #[test]
    fn should_keep_the_text_of_an_icon_refusal_without_a_category_as_unknown() {
        let event = WarningEvent::InstanceIconNotImported {
            cause: Arc::new(AssetError::NotAnImage),
        };

        assert_eq!(
            to_json(event)["warning"]["payload"]["cause"]["type"],
            "unknown"
        );
    }
}
