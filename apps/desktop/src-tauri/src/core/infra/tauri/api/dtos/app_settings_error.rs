use serde::Serialize;
use specta::Type;

use crate::features::settings::AppSettingsError;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppSettingsErrorDto {
    TransparentEffectRequired,

    #[serde(rename_all = "camelCase")]
    CanNotSetEffect {
        details: String,
    },

    #[serde(rename_all = "camelCase")]
    CanNotRecreateWindow {
        details: String,
    },

    SaveFailed,
}

impl From<AppSettingsError> for AppSettingsErrorDto {
    fn from(value: AppSettingsError) -> Self {
        match value {
            AppSettingsError::TransparentEffectRequired => Self::TransparentEffectRequired,
            AppSettingsError::CanNotSetEffect(msg) => Self::CanNotSetEffect { details: msg },
            AppSettingsError::CanNotRecreateWindow(msg) => {
                Self::CanNotRecreateWindow { details: msg }
            }
            AppSettingsError::SaveError => Self::SaveFailed,
        }
    }
}
