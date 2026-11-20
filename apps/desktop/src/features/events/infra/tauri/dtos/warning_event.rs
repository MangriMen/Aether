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
#[serde(
    tag = "code",
    content = "payload",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum WarningDto {
    InstanceIconNotImported {
        cause: CauseDto,
    },
    InstanceIconTooLarge {
        // Formatted before it crosses the boundary: no `u64` in bindings.
        limit: String,
    },
    InstanceIconNotSupported,
    /// The game ended while no launcher followed it, or while one that did not start it
    /// followed it and could not tell how it ended.
    PostExitHookSkipped {
        instance_id: String,
        instance_name: String,
    },
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
            WarningEvent::PostExitHookSkipped {
                instance_id,
                instance_name,
            } => Self::PostExitHookSkipped {
                instance_id: instance_id.to_string(),
                instance_name,
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
                "payload": {"cause": {"type": "io", "kind": "permission_denied"}},
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

    #[test]
    fn should_name_the_instance_when_its_post_exit_hook_was_skipped() {
        let skipped = WarningEvent::PostExitHookSkipped {
            instance_id: aether_core::features::instance::InstanceId::try_from("survival").unwrap(),
            instance_name: "Survival".to_owned(),
        };

        assert_eq!(
            to_json(skipped)["warning"],
            json!({
                "code": "POST_EXIT_HOOK_SKIPPED",
                "payload": {"instanceId": "survival", "instanceName": "Survival"},
            })
        );
    }
}
