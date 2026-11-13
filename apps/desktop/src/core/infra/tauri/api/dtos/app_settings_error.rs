use serde::Serialize;
use specta::Type;

use super::CauseDto;
use crate::features::settings::{AppSettingsError, AppSettingsStorageError};

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppSettingsErrorDto {
    CanNotSetEffect { cause: CauseDto },

    CanNotRecreateWindow { cause: CauseDto },

    Storage { cause: CauseDto },
}

impl From<AppSettingsError> for AppSettingsErrorDto {
    fn from(value: AppSettingsError) -> Self {
        match value {
            AppSettingsError::CanNotSetEffect(msg) => Self::CanNotSetEffect {
                cause: CauseDto::unknown(msg),
            },
            AppSettingsError::CanNotRecreateWindow(msg) => Self::CanNotRecreateWindow {
                cause: CauseDto::unknown(msg),
            },
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
    use serde_json::json;

    #[test]
    fn should_serialize_can_not_set_effect_with_an_unknown_cause() {
        let dto = AppSettingsErrorDto::from(AppSettingsError::CanNotSetEffect("boom".to_owned()));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "CAN_NOT_SET_EFFECT", "payload": {"cause": {"type": "unknown", "details": "boom"}}})
        );
    }

    #[test]
    fn should_map_database_storage_error_to_database_cause() {
        let dto = AppSettingsErrorDto::from(AppSettingsError::from(sqlx::Error::PoolTimedOut));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "STORAGE", "payload": {"cause": {"type": "database"}}})
        );
    }
}
