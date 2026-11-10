use crate::features::instance::{
    Instance, InstanceStorage,
    infra::{FsInstanceStorage, SqliteInstanceStorage},
};
use std::collections::HashSet;

use tracing::{info, warn};

/// Copies the legacy `instance.json` files into SQLite, whichever instances the table holds.
///
/// A copied file is set aside (`instance.json.migrated`), so the migration runs once per instance
/// and an instance the user removed is not brought back from it. The instances go in one
/// transaction, so a failure of the write itself stores none. One instance that cannot be stored
/// is skipped (its file stays, and it is shown as broken) rather than keeping the launcher from
/// starting.
///
/// An instance the table already holds is not copied over. Its file is set aside only when the
/// stored copy can be read: a record that cannot is left a file to be recovered from.
pub async fn migrate_instances_to_sqlite(
    fs_storage: &FsInstanceStorage,
    sqlite_storage: &SqliteInstanceStorage,
) -> crate::Result<()> {
    let old_instances = match fs_storage.list().await {
        Ok(instances) if !instances.is_empty() => instances,
        _ => return Ok(()),
    };

    let readable: HashSet<_> = sqlite_storage
        .list()
        .await?
        .iter()
        .map(|kept| kept.id().clone())
        .collect();
    let stored: HashSet<String> = sqlite_storage
        .stored_ids()
        .await?
        .into_iter()
        .map(|id| id.to_lowercase())
        .collect();

    // Files left by a build that kept them: SQLite holds those instances already.
    let held: Vec<_> = old_instances
        .iter()
        .filter(|old| readable.contains(old.id()))
        .collect();
    mark_migrated(fs_storage, held).await;

    let to_copy: Vec<Instance> = old_instances
        .into_iter()
        .filter(|old| !stored.contains(&old.id().as_str().to_lowercase()))
        .collect();
    if to_copy.is_empty() {
        return Ok(());
    }

    info!(
        "Legacy instances found (count: {}). Starting migration to SQLite...",
        to_copy.len()
    );

    let failed = sqlite_storage.upsert_many(&to_copy).await.map_err(|e| {
        warn!("Failed to migrate instances: {}", e);
        e
    })?;

    // An instance that could not be stored keeps its file, so it can be recovered by hand.
    for (instance, e) in &failed {
        warn!("Skipped instance {}: {}", instance.id(), e);
    }
    let saved = to_copy
        .iter()
        .filter(|old| !failed.iter().any(|(bad, _)| bad.id() == old.id()))
        .collect();
    mark_migrated(fs_storage, saved).await;

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
    use crate::features::minecraft::GameVersion;
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::*;
    use crate::features::instance::InstanceName;
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
                InstanceName::parse(id).unwrap(),
                GameVersion::from_static("1.21.1"),
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
    async fn should_skip_an_instance_that_cannot_be_saved_and_keep_its_file() {
        let (_temp, fs) = legacy_storage(&["a", "b", "c"]).await;
        let sqlite = SqliteInstanceStorage::in_memory().await;
        sqlx::query(
            "CREATE TRIGGER reject_c BEFORE INSERT ON instances WHEN NEW.id = 'c'
             BEGIN SELECT RAISE(ABORT, 'rejected'); END",
        )
        .execute(sqlite.pool())
        .await
        .unwrap();

        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();

        let saved: Vec<_> = sqlite.list().await.unwrap();
        assert_eq!(saved.len(), 2);
        let left: Vec<_> = fs.list().await.unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].id().as_str(), "c");
    }

    fn instance(id: &str) -> Instance {
        InstanceBuilder::new(
            InstanceId::try_from(id).unwrap(),
            InstanceName::parse(id).unwrap(),
            GameVersion::from_static("1.21.1"),
            ModLoader::Fabric,
        )
        .build()
    }

    #[tokio::test]
    async fn should_copy_a_legacy_instance_when_the_table_already_holds_others() {
        let (_temp, fs) = legacy_storage(&["old"]).await;
        let sqlite = SqliteInstanceStorage::in_memory().await;
        sqlite.upsert(&instance("new")).await.unwrap();

        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();

        let mut ids: Vec<_> = sqlite
            .list()
            .await
            .unwrap()
            .iter()
            .map(|kept| kept.id().to_string())
            .collect();
        ids.sort();
        assert_eq!(ids, ["new", "old"]);
        assert!(fs.list().await.unwrap().is_empty(), "its file is set aside");
    }

    #[tokio::test]
    async fn should_set_the_file_aside_and_keep_the_record_when_the_table_holds_the_instance() {
        let (_temp, fs) = legacy_storage(&["a"]).await;
        let sqlite = SqliteInstanceStorage::in_memory().await;
        let mut stored = instance("a");
        stored.rename(InstanceName::parse("renamed").unwrap());
        sqlite.upsert(&stored).await.unwrap();

        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();

        let kept = sqlite
            .get(&InstanceId::try_from("a").unwrap())
            .await
            .unwrap();
        assert_eq!(kept.name().as_str(), "renamed", "not overwritten");
        assert!(fs.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_keep_the_file_when_the_stored_record_of_the_instance_cannot_be_read() {
        let (_temp, fs) = legacy_storage(&["a"]).await;
        let sqlite = SqliteInstanceStorage::in_memory().await;
        sqlite.upsert(&instance("a")).await.unwrap();
        sqlx::query("UPDATE instances SET game_version = '../x' WHERE id = 'a'")
            .execute(sqlite.pool())
            .await
            .unwrap();

        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();

        assert_eq!(fs.list().await.unwrap().len(), 1, "a copy to recover from");
    }

    #[tokio::test]
    async fn should_not_take_in_a_file_when_its_folder_has_another_name_than_its_instance() {
        let (temp, fs) = legacy_storage(&["old"]).await;
        let location = LocationInfo::new(temp.path().join("settings"), temp.path().to_path_buf());
        let from = location.instance_metadata_file(&InstanceId::try_from("old").unwrap());
        let copy = location.instance_metadata_file(&InstanceId::try_from("copy").unwrap());
        std::fs::create_dir_all(copy.parent().unwrap()).unwrap();
        std::fs::copy(&from, &copy).unwrap();
        // The original is gone: the user removed that instance.
        std::fs::remove_file(&from).unwrap();
        let sqlite = SqliteInstanceStorage::in_memory().await;

        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();

        assert!(
            sqlite.list().await.unwrap().is_empty(),
            "nothing comes back"
        );
        assert!(copy.exists(), "the file is left where it is");
    }
}
