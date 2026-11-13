pub(crate) mod migrate_app_settings_to_sqlite;
pub(crate) mod sqlite_app_settings_storage;

pub use migrate_app_settings_to_sqlite::*;
pub use sqlite_app_settings_storage::*;

impl From<sqlx::Error> for crate::features::settings::AppSettingsError {
    fn from(err: sqlx::Error) -> Self {
        aether_core::shared::error::DatabaseError::from(err).into()
    }
}
