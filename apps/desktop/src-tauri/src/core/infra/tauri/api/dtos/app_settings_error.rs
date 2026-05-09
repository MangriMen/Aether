use serde::Serialize;
use specta::Type;

use crate::features::settings::AppSettingsError;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppSettingsErrorDto {
    #[serde(rename_all = "camelCase")]
    CanNotSetEffect { details: String },

    #[serde(rename_all = "camelCase")]
    CanNotRecreateWindow { details: String },

    #[serde(rename_all = "camelCase")]
    Storage { details: String },
}

impl From<AppSettingsError> for AppSettingsErrorDto {
    fn from(value: AppSettingsError) -> Self {
        match value {
            AppSettingsError::CanNotSetEffect(msg) => Self::CanNotSetEffect { details: msg },
            AppSettingsError::CanNotRecreateWindow(msg) => {
                Self::CanNotRecreateWindow { details: msg }
            }
            AppSettingsError::Storage(msg) => Self::Storage { details: msg },
        }
    }
}
