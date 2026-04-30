use crate::features::settings::SettingsError;

impl From<sqlx::Error> for SettingsError {
    fn from(value: sqlx::Error) -> Self {
        SettingsError::Storage(value.to_string())
    }
}
