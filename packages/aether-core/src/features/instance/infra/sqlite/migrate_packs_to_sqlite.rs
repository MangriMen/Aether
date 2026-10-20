use crate::features::instance::{
    InstanceStorage, PackStorage,
    infra::{FsPackStorage, SqlitePackStorage},
};
use tracing::{info, warn};

pub async fn migrate_packs_to_sqlite(
    instance_storage: &dyn InstanceStorage,
    fs_storage: &FsPackStorage,
    sqlite_storage: &SqlitePackStorage,
) -> crate::Result<()> {
    let instances = instance_storage.list().await?;

    for instance in instances {
        let instance_id = &instance.id();

        let Ok(pack) = fs_storage.get_pack(instance_id).await else {
            continue;
        };

        // The pack row outlives its files, so a pack the user emptied is not migrated again.
        // A failed lookup counts as migrated: leaving the legacy file unread is the safe side.
        let already_migrated = sqlite_storage.has_pack(instance_id).await.unwrap_or(true);
        if already_migrated {
            continue;
        }

        info!(
            "Legacy pack found for instance {}. Starting migration to SQLite...",
            instance_id
        );

        let mut content_paths = Vec::new();
        let mut pack_files = Vec::new();
        for entry in pack.files {
            let content_path = entry.file;
            match fs_storage.get_pack_file(instance_id, &content_path).await {
                Ok(pack_file) => {
                    content_paths.push(content_path);
                    pack_files.push(pack_file);
                }
                // An unreadable legacy file stays unreadable, so it is skipped rather than retried.
                Err(e) => {
                    warn!(
                        "Failed to read legacy pack file {} for instance {}: {}",
                        content_path, instance_id, e
                    );
                }
            }
        }

        // One transaction: a pack row only appears together with all of its readable files,
        // because `has_pack` stops the next start from migrating the rest.
        sqlite_storage
            .update_pack_file_many(instance_id, &content_paths, &pack_files)
            .await?;

        info!(
            "Migration of pack for instance {} to SQLite completed successfully.",
            instance_id
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use tempfile::TempDir;

    use super::*;
    use crate::features::{
        instance::{
            Instance, InstanceBuilder, InstanceError, InstanceId, Pack, PackEntry, PackFile,
        },
        minecraft::ModLoader,
        settings::LocationInfo,
    };

    struct MemInstanceStorage(Mutex<Vec<Instance>>);

    #[async_trait]
    impl InstanceStorage for MemInstanceStorage {
        async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
            Ok(self.0.lock().unwrap().clone())
        }
        async fn get(&self, _: &InstanceId) -> Result<Instance, InstanceError> {
            unimplemented!()
        }
        async fn upsert(&self, _: &Instance) -> Result<(), InstanceError> {
            unimplemented!()
        }
        async fn remove(&self, _: &InstanceId) -> Result<(), InstanceError> {
            unimplemented!()
        }
    }

    async fn legacy_pack(fs: &FsPackStorage, instance_id: &InstanceId, path: &str, hash: &str) {
        fs.update_pack(
            instance_id,
            &Pack {
                files: vec![PackEntry { file: path.into() }],
            },
        )
        .await
        .unwrap();
        fs.update_pack_file(
            instance_id,
            path,
            &PackFile::from_hash("a.jar".into(), hash.into()),
        )
        .await
        .unwrap();
    }

    fn setup() -> (TempDir, MemInstanceStorage, FsPackStorage, InstanceId) {
        let temp = TempDir::new().unwrap();
        let location = Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        ));
        let id = InstanceId::try_from("inst").unwrap();
        let instance = InstanceBuilder::new(
            id.clone(),
            "Inst".into(),
            "1.21.1".into(),
            ModLoader::Fabric,
        )
        .build();
        (
            temp,
            MemInstanceStorage(Mutex::new(vec![instance])),
            FsPackStorage::new(location),
            id,
        )
    }

    #[tokio::test]
    async fn should_migrate_legacy_pack_when_sqlite_is_empty() {
        let (_temp, instances, fs, id) = setup();
        legacy_pack(&fs, &id, "mods/a.jar", "legacy").await;
        let sqlite = SqlitePackStorage::in_memory().await;

        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();

        let migrated = sqlite.get_pack_file(&id, "mods/a.jar").await.unwrap();
        assert_eq!(migrated.hash, "legacy");
    }

    #[tokio::test]
    async fn should_not_overwrite_pack_when_already_migrated() {
        let (_temp, instances, fs, id) = setup();
        legacy_pack(&fs, &id, "mods/a.jar", "legacy").await;
        let sqlite = SqlitePackStorage::in_memory().await;
        sqlite
            .update_pack_file(
                &id,
                "mods/a.jar",
                &PackFile::from_hash("a.jar".into(), "current".into()),
            )
            .await
            .unwrap();

        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();

        let kept = sqlite.get_pack_file(&id, "mods/a.jar").await.unwrap();
        assert_eq!(kept.hash, "current");
    }

    #[tokio::test]
    async fn should_migrate_readable_files_in_one_go_when_one_legacy_file_is_missing() {
        let (_temp, instances, fs, id) = setup();
        legacy_pack(&fs, &id, "mods/a.jar", "legacy").await;
        fs.update_pack(
            &id,
            &Pack {
                files: vec![
                    PackEntry {
                        file: "mods/missing.jar".into(),
                    },
                    PackEntry {
                        file: "mods/a.jar".into(),
                    },
                ],
            },
        )
        .await
        .unwrap();
        let sqlite = SqlitePackStorage::in_memory().await;

        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();

        let migrated = sqlite.get_pack(&id).await.unwrap();
        assert_eq!(migrated.files.len(), 1);
        assert_eq!(migrated.files[0].file, "mods/a.jar");
    }

    #[tokio::test]
    async fn should_not_bring_legacy_files_back_when_every_migrated_file_was_removed() {
        let (_temp, instances, fs, id) = setup();
        legacy_pack(&fs, &id, "mods/a.jar", "legacy").await;
        let sqlite = SqlitePackStorage::in_memory().await;
        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();
        sqlite.remove_pack_file(&id, "mods/a.jar").await.unwrap();

        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();

        assert!(sqlite.get_pack(&id).await.unwrap().files.is_empty());
    }
}
