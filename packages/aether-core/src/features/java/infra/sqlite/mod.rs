mod migrate_java_to_sqlite;
mod sqlite_java_storage;

pub use migrate_java_to_sqlite::migrate_java_to_sqlite;
pub use sqlite_java_storage::SqliteJavaStorage;

impl From<sqlx::Error> for crate::features::java::JavaDomainError {
    fn from(err: sqlx::Error) -> Self {
        Self::Storage(err.into())
    }
}
