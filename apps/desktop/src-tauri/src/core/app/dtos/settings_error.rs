use aether_core::features::settings::SettingsError;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettingsErrorDto {
    NotFound,

    StorageFailure,
}

impl From<&SettingsError> for SettingsErrorDto {
    fn from(value: &SettingsError) -> Self {
        match value {
            SettingsError::NotFound => SettingsErrorDto::NotFound,
            SettingsError::StorageFailure(_) => SettingsErrorDto::StorageFailure,
        }
    }
}
