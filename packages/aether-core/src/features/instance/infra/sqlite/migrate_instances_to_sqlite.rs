use crate::features::instance::{
    Instance, InstanceStorage,
    infra::{FsInstanceStorage, SqliteInstanceStorage},
};
use tracing::{info, warn};

/// Copies the legacy `instance.json` files into SQLite, and only into an empty table.
///
/// A copied file is set aside (`instance.json.migrated`), so the migration runs once per instance
/// and a table the user emptied is not refilled from it. All instances go in one transaction: a
/// table that kept only the first ones would be skipped on the next start and the rest lost.
pub async fn migrate_instances_to_sqlite(
    fs_storage: &FsInstanceStorage,
    sqlite_storage: &SqliteInstanceStorage,
) -> crate::Result<()> {
    let old_instances = match fs_storage.list().await {
        Ok(instances) if !instances.is_empty() => instances,
        _ => return Ok(()),
    };

    let existing_sqlite = sqlite_storage.list().await?;
    if !existing_sqlite.is_empty() {
        // Files left by a build that kept them: SQLite holds those instances already.
        let held: Vec<_> = old_instances
            .iter()
            .filter(|old| existing_sqlite.iter().any(|kept| kept.id() == old.id()))
            .collect();
        mark_migrated(fs_storage, held).await;
        return Ok(());
    }

    info!(
        "Legacy instances found (count: {}). Starting migration to SQLite...",
        old_instances.len()
    );

    if let Err(e) = sqlite_storage.upsert_many(&old_instances).await {
        warn!("Failed to migrate instances: {}", e);
        return Err(e.into());
    }

    mark_migrated(fs_storage, old_instances.iter().collect()).await;

    info!("Migration of instances to SQLite completed successfully.");

    Ok(())
}

async fn mark_migrated(fs_storage: &FsInstanceStorage, instances: Vec<&Instance>) {
    for instance in instances {
        if let Err(e) = fs_storage.mark_migrated(instance.id()).await {
            warn!(
                "Failed to set aside the legacy file of {}: {}",
                instance.id(),
                e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::*;
    use crate::features::{
        instance::{InstanceBuilder, InstanceId},
        minecraft::ModLoader,
        settings::LocationInfo,
    };

    async fn legacy_storage(ids: &[&str]) -> (TempDir, FsInstanceStorage) {
        let temp = TempDir::new().unwrap();
        let location = Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        ));
        let fs = FsInstanceStorage::new(location);
        for id in ids {
            let instance = InstanceBuilder::new(
                InstanceId::try_from(*id).unwrap(),
                (*id).to_owned(),
                "1.21.1".into(),
                ModLoader::Fabric,
            )
            .build();
            fs.upsert(&instance).await.unwrap();
        }
        (temp, fs)
    }

    #[tokio::test]
    async fn should_copy_every_legacy_instance_into_an_empty_table() {
        let (_temp, fs) = legacy_storage(&["a", "b"]).await;
        let sqlite = SqliteInstanceStorage::in_memory().await;

        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();

        assert_eq!(sqlite.list().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn should_not_copy_an_instance_again_when_the_migration_runs_twice() {
        let (_temp, fs) = legacy_storage(&["a", "b"]).await;
        let sqlite = SqliteInstanceStorage::in_memory().await;

        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();
        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();

        assert_eq!(sqlite.list().await.unwrap().len(), 2);
        assert!(fs.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_leave_table_empty_and_keep_legacy_files_when_one_instance_cannot_be_saved() {
        let (_temp, fs) = legacy_storage(&["a", "b", "c"]).await;
        let sqlite = SqliteInstanceStorage::in_memory().await;
        sqlx::query(
            "CREATE TRIGGER reject_c BEFORE INSERT ON instances WHEN NEW.id = 'c'
             BEGIN SELECT RAISE(ABORT, 'disk full'); END",
        )
        .execute(sqlite.pool())
        .await
        .unwrap();

        assert!(migrate_instances_to_sqlite(&fs, &sqlite).await.is_err());

        assert!(sqlite.list().await.unwrap().is_empty());
        assert_eq!(fs.list().await.unwrap().len(), 3);
    }
}
