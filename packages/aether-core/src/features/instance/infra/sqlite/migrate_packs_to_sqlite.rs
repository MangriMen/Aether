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

        if !fs_storage.has_legacy_pack(instance_id) {
            continue;
        }
        let Ok(pack) = fs_storage.get_pack(instance_id).await else {
            continue;
        };

        // The pack row outlives its files, so a pack the user emptied is not migrated again.
        // A failed lookup counts as migrated: leaving the legacy file unread is the safe side.
        match sqlite_storage.has_pack(instance_id).await {
            Ok(true) => {
                set_aside(fs_storage, instance_id).await;
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                warn!("Failed to check the migrated pack of instance {instance_id}: {e}");
                continue;
            }
        }

        info!(
            "Legacy pack found for instance {}. Starting migration to SQLite...",
            instance_id
        );

        let mut content_paths = Vec::new();
        let mut pack_files = Vec::new();
        for entry in pack.files {
            let content_path = entry.file;
            match fs_storage
                .get_legacy_pack_file(instance_id, &content_path)
                .await
            {
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
        // A failed write leaves no pack row, so the next start tries this pack again.
        if let Err(e) = sqlite_storage
            .update_pack_file_many(instance_id, &content_paths, &pack_files)
            .await
        {
            warn!("Failed to migrate pack of instance {instance_id}: {e}");
            continue;
        }
        set_aside(fs_storage, instance_id).await;

        info!(
            "Migration of pack for instance {} to SQLite completed successfully.",
            instance_id
        );
    }

    Ok(())
}

/// The legacy pack is read once: set aside, it cannot be migrated again.
async fn set_aside(
    fs_storage: &FsPackStorage,
    instance_id: &crate::features::instance::InstanceId,
) {
    if let Err(e) = fs_storage.mark_migrated(instance_id).await {
        warn!("Failed to set aside the legacy pack of instance {instance_id}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use crate::features::minecraft::GameVersion;
    use std::sync::{Arc, Mutex};

    fn sha1_hash(c: char) -> crate::shared::hash::FileHash {
        crate::shared::hash::FileHash::new(
            crate::shared::hash::HashAlgorithm::Sha1,
            c.to_string().repeat(40),
        )
        .unwrap()
    }

    use async_trait::async_trait;
    use tempfile::TempDir;

    use super::*;
    use crate::features::instance::InstanceName;
    use crate::features::{
        instance::{
            ContentPath, Instance, InstanceBuilder, InstanceError, InstanceId, Pack, PackEntry,
            PackFile,
        },
        minecraft::ModLoader,
        settings::LocationInfo,
    };

    fn cp(path: &str) -> ContentPath {
        ContentPath::parse(path).unwrap()
    }

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
        async fn update(
            &self,
            id: &InstanceId,
            change: crate::features::instance::InstanceChange<'_>,
        ) -> Result<Instance, InstanceError> {
            let mut instance = self.get(id).await?;
            if change(&mut instance)? {
                self.upsert(&instance).await?;
            }
            Ok(instance)
        }
        async fn remove(&self, _: &InstanceId) -> Result<(), InstanceError> {
            unimplemented!()
        }
    }

    async fn legacy_pack(fs: &FsPackStorage, instance_id: &InstanceId, path: &str, hash: char) {
        fs.update_pack(
            instance_id,
            &Pack {
                files: vec![PackEntry { file: cp(path) }],
            },
        )
        .await
        .unwrap();
        fs.update_pack_file(
            instance_id,
            &cp(path),
            &PackFile::from_hash("a.jar".into(), sha1_hash(hash)),
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
            InstanceName::parse("Inst").unwrap(),
            GameVersion::from_static("1.21.1"),
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
        legacy_pack(&fs, &id, "mods/a.jar", 'e').await;
        let sqlite = SqlitePackStorage::in_memory().await;

        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();

        let migrated = sqlite.get_pack_file(&id, &cp("mods/a.jar")).await.unwrap();
        assert_eq!(migrated.hash, sha1_hash('e'));
    }

    #[tokio::test]
    async fn should_set_the_legacy_pack_aside_when_it_was_migrated() {
        let (_temp, instances, fs, id) = setup();
        legacy_pack(&fs, &id, "mods/a.jar", 'e').await;
        let sqlite = SqlitePackStorage::in_memory().await;

        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();

        assert!(!fs.has_legacy_pack(&id));
    }

    #[tokio::test]
    async fn should_not_overwrite_pack_when_already_migrated() {
        let (_temp, instances, fs, id) = setup();
        legacy_pack(&fs, &id, "mods/a.jar", 'e').await;
        let sqlite = SqlitePackStorage::in_memory().await;
        sqlite
            .update_pack_file(
                &id,
                &cp("mods/a.jar"),
                &PackFile::from_hash("a.jar".into(), sha1_hash('c')),
            )
            .await
            .unwrap();

        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();

        let kept = sqlite.get_pack_file(&id, &cp("mods/a.jar")).await.unwrap();
        assert_eq!(kept.hash, sha1_hash('c'));
    }

    #[tokio::test]
    async fn should_migrate_readable_files_in_one_go_when_one_legacy_file_is_missing() {
        let (_temp, instances, fs, id) = setup();
        legacy_pack(&fs, &id, "mods/a.jar", 'e').await;
        fs.update_pack(
            &id,
            &Pack {
                files: vec![
                    PackEntry {
                        file: cp("mods/missing.jar"),
                    },
                    PackEntry {
                        file: cp("mods/a.jar"),
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
        assert_eq!(migrated.files[0].file, cp("mods/a.jar"));
    }

    #[tokio::test]
    async fn should_not_bring_legacy_files_back_when_every_migrated_file_was_removed() {
        let (_temp, instances, fs, id) = setup();
        legacy_pack(&fs, &id, "mods/a.jar", 'e').await;
        let sqlite = SqlitePackStorage::in_memory().await;
        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();
        sqlite
            .remove_pack_file(&id, &cp("mods/a.jar"))
            .await
            .unwrap();

        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();

        assert!(sqlite.get_pack(&id).await.unwrap().files.is_empty());
    }

    /// A legacy pack with one record, `mods/a.jar`, whose stored hash is invalid and which links
    /// to a provider. `on_disk` is where the jar is put (relative to the instance), if anywhere.
    async fn legacy_pack_with_invalid_hash(
        on_disk: Option<&str>,
    ) -> (
        SqlitePackStorage,
        MemInstanceStorage,
        FsPackStorage,
        InstanceId,
        TempDir,
    ) {
        let (temp, instances, fs, id) = setup();
        let provider: crate::features::instance::ProviderId = "aether#modrinth".parse().unwrap();
        let mut record = PackFile::from_hash("a.jar".into(), sha1_hash('e'));
        record.update_provider_id = Some(provider);
        fs.update_pack(
            &id,
            &Pack {
                files: vec![PackEntry {
                    file: cp("mods/a.jar"),
                }],
            },
        )
        .await
        .unwrap();
        fs.update_pack_file(&id, &cp("mods/a.jar"), &record)
            .await
            .unwrap();
        let location = LocationInfo::new(temp.path().join("settings"), temp.path().to_path_buf());
        let record_path = location.instance_pack_dir(&id).join("mods").join("a.toml");
        let stored = tokio::fs::read_to_string(&record_path).await.unwrap();
        tokio::fs::write(&record_path, stored.replace(&"e".repeat(40), "bad"))
            .await
            .unwrap();
        if let Some(file) = on_disk {
            let jar = location.instance_dir(&id).join(file);
            tokio::fs::create_dir_all(jar.parent().unwrap())
                .await
                .unwrap();
            tokio::fs::write(jar, b"jar").await.unwrap();
        }
        let sqlite = SqlitePackStorage::in_memory().await;
        migrate_packs_to_sqlite(&instances, &fs, &sqlite)
            .await
            .unwrap();
        (sqlite, instances, fs, id, temp)
    }

    const JAR_SHA1: &str = "f92e777f4341930bad9b2422283c4680d00dbc06";

    #[tokio::test]
    async fn should_keep_the_provider_link_and_rehash_when_a_legacy_record_has_an_invalid_hash() {
        let (sqlite, _instances, _fs, id, _temp) =
            legacy_pack_with_invalid_hash(Some("mods/a.jar")).await;

        let migrated = sqlite.get_pack_file(&id, &cp("mods/a.jar")).await.unwrap();
        assert_eq!(
            migrated.update_provider_id,
            Some("aether#modrinth".parse().unwrap())
        );
        assert_eq!(migrated.hash.hex(), JAR_SHA1);
    }

    #[tokio::test]
    async fn should_rehash_the_disabled_copy_when_a_legacy_record_has_an_invalid_hash() {
        let (sqlite, _instances, _fs, id, _temp) =
            legacy_pack_with_invalid_hash(Some("mods/a.jar.disabled")).await;

        let migrated = sqlite.get_pack_file(&id, &cp("mods/a.jar")).await.unwrap();
        assert_eq!(migrated.hash.hex(), JAR_SHA1);
    }

    #[tokio::test]
    async fn should_skip_the_record_when_a_legacy_record_has_an_invalid_hash_and_no_file() {
        let (sqlite, _instances, _fs, id, _temp) = legacy_pack_with_invalid_hash(None).await;

        assert!(sqlite.get_pack_file(&id, &cp("mods/a.jar")).await.is_err());
    }
}
