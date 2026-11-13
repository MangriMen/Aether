mod migrate_instances_to_sqlite;
mod migrate_packs_to_sqlite;
mod sql_instance;
mod sql_pack_info;
mod sqlite_instance_storage;
mod sqlite_pack_storage;

pub use migrate_instances_to_sqlite::migrate_instances_to_sqlite;
pub use migrate_packs_to_sqlite::migrate_packs_to_sqlite;
pub use sqlite_instance_storage::SqliteInstanceStorage;
pub use sqlite_pack_storage::SqlitePackStorage;

impl From<sqlx::Error> for crate::features::instance::StorageError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::features::instance::{InstanceError, StorageError};

    #[test]
    fn should_keep_a_database_failure_as_its_category() {
        let err = InstanceError::storage(sqlx::Error::RowNotFound);

        assert!(matches!(
            err,
            InstanceError::Storage(StorageError::Database(_))
        ));
    }
}
