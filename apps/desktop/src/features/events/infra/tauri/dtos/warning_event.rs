use aether_core::features::events::WarningEvent;
use serde::Serialize;
use specta::Type;
use tauri_specta::Event;

use aether_core::shared::cache::AssetError;

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
// The codes are the translation keys; the warnings of other kinds will not share the prefix.
#[allow(clippy::enum_variant_names)]
pub enum WarningDto {
    InstanceIconNotImported {
        cause: CauseDto,
    },
    InstanceIconTooLarge {
        // Formatted before it crosses the boundary: no `u64` in bindings.
        limit: String,
    },
    InstanceIconNotSupported,
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
            WarningEvent::InstanceIconNotImported { cause } => match cause.as_ref() {
                AssetError::TooLarge { limit } => Self::InstanceIconTooLarge {
                    limit: limit.to_string(),
                },
                AssetError::NotAnImage => Self::InstanceIconNotSupported,
                AssetError::ReadSource(err) | AssetError::Write(err) | AssetError::Remove(err) => {
                    Self::InstanceIconNotImported { cause: err.into() }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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
    fn should_name_why_the_icon_was_refused_by_a_code_of_its_own() {
        let too_large = WarningEvent::InstanceIconNotImported {
            cause: Arc::new(AssetError::TooLarge { limit: 4096 }),
        };
        let not_an_image = WarningEvent::InstanceIconNotImported {
            cause: Arc::new(AssetError::NotAnImage),
        };

        assert_eq!(
            to_json(too_large)["warning"],
            json!({"code": "INSTANCE_ICON_TOO_LARGE", "payload": {"limit": "4096"}})
        );
        assert_eq!(
            to_json(not_an_image)["warning"],
            json!({"code": "INSTANCE_ICON_NOT_SUPPORTED"})
        );
    }
}
