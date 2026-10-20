use crate::features::instance::{InstanceId, InstanceStorage, infra::FsInstanceStorage};
use tracing::{info, warn};

/// Copies the legacy `instance.json` files into SQLite, and only into an empty table.
///
/// The legacy files stay as a backup, so a table the user emptied must not be refilled
/// from them. For the same reason a failed run is undone: a table that kept the first
/// instances would be skipped on the next start and the rest would be lost.
pub async fn migrate_instances_to_sqlite(
    fs_storage: &FsInstanceStorage,
    sqlite_storage: &dyn InstanceStorage,
) -> crate::Result<()> {
    let old_instances = match fs_storage.list().await {
        Ok(instances) if !instances.is_empty() => instances,
        _ => return Ok(()),
    };

    let existing_sqlite = sqlite_storage.list().await?;
    if !existing_sqlite.is_empty() {
        return Ok(());
    }

    info!(
        "Legacy instances found (count: {}). Starting migration to SQLite...",
        old_instances.len()
    );

    let mut migrated: Vec<InstanceId> = Vec::new();
    for instance in old_instances {
        if let Err(e) = sqlite_storage.upsert(&instance).await {
            warn!("Failed to migrate instance {}: {}", instance.id(), e);
            roll_back(sqlite_storage, &migrated).await;
            return Err(e.into());
        }
        migrated.push(instance.id().clone());
    }

    info!("Migration of instances to SQLite completed successfully.");

    Ok(())
}

async fn roll_back(sqlite_storage: &dyn InstanceStorage, migrated: &[InstanceId]) {
    for id in migrated {
        if let Err(e) = sqlite_storage.remove(id).await {
            warn!("Failed to undo the migration of instance {}: {}", id, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use tempfile::TempDir;

    use super::*;
    use crate::features::{
        instance::{Instance, InstanceBuilder, InstanceError},
        minecraft::ModLoader,
        settings::LocationInfo,
    };

    #[derive(Default)]
    struct FakeStorage {
        stored: Mutex<Vec<InstanceId>>,
        failing: Option<InstanceId>,
    }

    #[async_trait]
    impl InstanceStorage for FakeStorage {
        async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
            Ok(Vec::new())
        }
        async fn get(&self, _: &InstanceId) -> Result<Instance, InstanceError> {
            unimplemented!()
        }
        async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
            if self.failing.as_ref() == Some(instance.id()) {
                return Err(InstanceError::Storage("disk full".into()));
            }
            self.stored.lock().unwrap().push(instance.id().clone());
            Ok(())
        }
        async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError> {
            self.stored.lock().unwrap().retain(|stored| stored != id);
            Ok(())
        }
    }

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
        let sqlite = FakeStorage::default();

        migrate_instances_to_sqlite(&fs, &sqlite).await.unwrap();

        assert_eq!(sqlite.stored.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn should_report_failure_and_leave_table_empty_when_one_instance_cannot_be_saved() {
        let (_temp, fs) = legacy_storage(&["a", "b", "c"]).await;
        let sqlite = FakeStorage {
            failing: Some(InstanceId::try_from("c").unwrap()),
            ..FakeStorage::default()
        };

        let result = migrate_instances_to_sqlite(&fs, &sqlite).await;

        assert!(result.is_err());
        assert!(sqlite.stored.lock().unwrap().is_empty());
    }
}
