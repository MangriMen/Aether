use serde::Serialize;
use specta::Type;

use super::{CauseDto, WindowErrorDto};
use crate::features::settings::{AppSettingsError, AppSettingsStorageError};

#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppSettingsErrorDto {
    /// `reason` is what the window manager refused with, translated as a reason.
    CanNotSetEffect {
        reason: WindowErrorDto,
    },

    CanNotRecreateWindow {
        reason: WindowErrorDto,
    },

    Storage {
        cause: CauseDto,
    },
}

impl From<AppSettingsError> for AppSettingsErrorDto {
    fn from(value: AppSettingsError) -> Self {
        match value {
            AppSettingsError::CanNotSetEffect(err) => Self::CanNotSetEffect { reason: err.into() },
            AppSettingsError::CanNotRecreateWindow(err) => {
                Self::CanNotRecreateWindow { reason: err.into() }
            }
            AppSettingsError::Storage(err) => Self::Storage {
                cause: match &err {
                    AppSettingsStorageError::Io(err) => err.into(),
                    AppSettingsStorageError::Database(err) => err.into(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{WindowError, WindowLabel};
    use serde_json::json;

    #[test]
    fn should_serialize_can_not_set_effect_with_the_window_error_as_its_reason() {
        let dto = AppSettingsErrorDto::from(AppSettingsError::CanNotSetEffect(
            WindowError::PlatformNotSupported,
        ));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "CAN_NOT_SET_EFFECT", "payload": {"reason": {"code": "PLATFORM_NOT_SUPPORTED"}}})
        );
    }

    #[test]
    fn should_serialize_can_not_recreate_window_with_the_window_error_as_its_reason() {
        let dto = AppSettingsErrorDto::from(AppSettingsError::CanNotRecreateWindow(
            WindowError::NotFound {
                label: WindowLabel::Main,
            },
        ));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "CAN_NOT_RECREATE_WINDOW", "payload": {"reason": {"code": "NOT_FOUND", "payload": {"label": "main"}}}})
        );
    }

    #[test]
    fn should_map_database_storage_error_to_database_cause() {
        let dto = AppSettingsErrorDto::from(AppSettingsError::from(
            aether_core::shared::error::DatabaseError::new("pool timed out"),
        ));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "STORAGE", "payload": {"cause": {"type": "database"}}})
        );
    }
}
