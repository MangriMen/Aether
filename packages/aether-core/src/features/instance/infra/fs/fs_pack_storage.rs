use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::{
    features::{
        instance::{
            ContentId, ContentPath, InstanceError, InstanceId, Pack, PackEntry, PackFile,
            PackStorage, ProviderId, app::DISABLED_SUFFIX,
        },
        settings::LocationInfo,
    },
    shared::{
        hash::{FileHash, InvalidFileHash, infra::sha1_hash_async},
        io::infra::{
            ensure_read_toml_async, read_async, read_toml_async, remove_dir_all, remove_file,
            rename, write_toml_async,
        },
    },
};

use super::pack_file::{PackFileV1, PackFileV2, PackV2};
use crate::features::instance::infra::{
    stored_content_path::parse_stored_content_path, stored_hash::parse_stored_hash,
};

pub struct FsPackStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPackStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    /// Whether a legacy pack file is there. `get_pack` would create an empty one.
    pub fn has_legacy_pack(&self, instance_id: &InstanceId) -> bool {
        self.get_pack_path(instance_id).exists()
    }

    /// Sets the legacy pack directory aside as `pack.migrated`, so that it is read once and a
    /// later start does not bring the pack back from it.
    pub async fn mark_migrated(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let dir = self.location_info.instance_pack_dir(instance_id);
        if !dir.exists() {
            return Ok(());
        }
        let target = dir.with_file_name("pack.migrated");
        if target.exists() {
            remove_dir_all(&target)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }
        rename(&dir, &target)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }

    /// Reads a legacy record for migration: one with an invalid hash keeps its other fields (the
    /// provider link, update info, owner) and gets the sha1 of the file on disk, if it is there.
    pub async fn get_legacy_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<PackFile, InstanceError> {
        self.read_pack_file(instance_id, content_path, true)
            .await?
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }

    /// The record, or why its hash is not one. With `rehash`, an invalid hash is replaced by the
    /// sha1 of the file on disk.
    async fn read_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        rehash: bool,
    ) -> Result<Result<PackFile, InvalidFileHash>, InstanceError> {
        let path = self.get_pack_file_path(instance_id, content_path);

        let v2: Result<PackFileV2, _> = read_toml_async(&path).await;
        let (file_name, stored_hash) = if let Ok(dto) = &v2 {
            (dto.file_name.clone(), dto.hash.clone())
        } else {
            let v1: PackFileV1 = read_toml_async(&path)
                .await
                .map_err(|e| InstanceError::Storage(e.to_string()))?;
            (v1.file_name, v1.hash)
        };

        let hash = match parse_stored_hash(&file_name, &stored_hash) {
            Ok(hash) => hash,
            Err(err) if rehash => match self.sha1_on_disk(instance_id, content_path).await {
                Some(hash) => hash,
                None => return Ok(Err(err)),
            },
            Err(err) => return Ok(Err(err)),
        };

        if let Ok(dto) = v2 {
            return Ok(Ok(dto.into_pack_file(hash)));
        }

        let v1: PackFileV1 = read_toml_async(&path)
            .await
            .map_err(|e| InstanceError::Storage(e.to_string()))?;
        let migrated = v1.into_pack_file(hash);
        let _ = self
            .update_pack_file(instance_id, content_path, &migrated)
            .await;
        Ok(Ok(migrated))
    }

    async fn sha1_on_disk(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Option<FileHash> {
        let enabled = self
            .location_info
            .instance_dir(instance_id)
            .join(content_path.to_path_buf());
        let mut disabled = enabled.clone().into_os_string();
        disabled.push(DISABLED_SUFFIX);

        for path in [enabled, PathBuf::from(disabled)] {
            let Ok(content) = read_async(&path).await else {
                continue;
            };
            match sha1_hash_async(content).await {
                Ok(hash) => {
                    tracing::warn!(
                        "Rehashed \"{content_path}\" of \"{instance_id}\": its stored hash is invalid"
                    );
                    return Some(hash);
                }
                Err(err) => tracing::warn!("Failed to hash \"{content_path}\": {err}"),
            }
        }
        None
    }

    fn get_pack_path(&self, instance_id: &InstanceId) -> PathBuf {
        self.location_info.instance_pack(instance_id)
    }

    fn get_pack_file_path(&self, instance_id: &InstanceId, content_path: &ContentPath) -> PathBuf {
        self.location_info
            .instance_pack_dir(instance_id)
            .join(content_path.to_path_buf())
            .with_extension("toml")
    }
}

#[async_trait]
impl PackStorage for FsPackStorage {
    async fn get_pack(&self, instance_id: &InstanceId) -> Result<Pack, InstanceError> {
        let dto: PackV2 = ensure_read_toml_async(&self.get_pack_path(instance_id))
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;

        // An entry that fails the check can never name a file of an instance: it is dropped
        // rather than failing the read of the whole pack.
        Ok(Pack {
            files: dto
                .files
                .into_iter()
                .filter_map(|e| {
                    parse_stored_content_path(instance_id, &e.file).map(|file| PackEntry { file })
                })
                .collect(),
        })
    }

    async fn update_pack(
        &self,
        instance_id: &InstanceId,
        pack: &Pack,
    ) -> Result<(), InstanceError> {
        let dto = PackV2 {
            files: pack
                .files
                .iter()
                .map(|e| super::pack_file::PackEntryV2 {
                    file: e.file.as_str().to_owned(),
                })
                .collect(),
        };
        write_toml_async(&self.get_pack_path(instance_id), &dto)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        Ok(())
    }

    async fn get_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<PackFile, InstanceError> {
        self.read_pack_file(instance_id, content_path, false)
            .await?
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }

    async fn rehash_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        hash: &FileHash,
    ) -> Result<Option<PackFile>, InstanceError> {
        let path = self.get_pack_file_path(instance_id, content_path);
        if !path.exists() {
            return Ok(None);
        }

        let v2: Result<PackFileV2, _> = read_toml_async(&path).await;
        let pack_file = if let Ok(dto) = v2 {
            dto.into_pack_file(hash.clone())
        } else {
            let v1: PackFileV1 = read_toml_async(&path)
                .await
                .map_err(|e| InstanceError::Storage(e.to_string()))?;
            v1.into_pack_file(hash.clone())
        };
        self.update_pack_file(instance_id, content_path, &pack_file)
            .await?;
        Ok(Some(pack_file))
    }

    async fn find_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<Option<PackFile>, InstanceError> {
        Ok(self
            .read_pack_file(instance_id, content_path, false)
            .await?
            .ok())
    }

    async fn get_pack_files(
        &self,
        instance_id: &InstanceId,
    ) -> Result<HashMap<ContentPath, PackFile>, InstanceError> {
        let mut files = HashMap::new();

        for entry in self.get_pack(instance_id).await?.files {
            // A record with an invalid hash is left out, like a file nothing recorded.
            if let Ok(pack_file) = self.read_pack_file(instance_id, &entry.file, false).await? {
                files.insert(entry.file, pack_file);
            }
        }

        Ok(files)
    }

    async fn find_by_provider_id(
        &self,
        instance_id: &InstanceId,
        provider_id: &ProviderId,
        content_id: &ContentId,
    ) -> Result<Option<PackFile>, InstanceError> {
        let pack = self.get_pack(instance_id).await?;

        for entry in pack.files {
            if let Ok(pack_file) = self.get_pack_file(instance_id, &entry.file).await
                && let Some(update_map) = &pack_file.update
                && let Some(info) = update_map.get(provider_id)
                && &info.content_id == content_id
            {
                return Ok(Some(pack_file));
            }
        }

        Ok(None)
    }

    async fn update_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        self.update_pack_file_many(
            instance_id,
            std::slice::from_ref(content_path),
            std::slice::from_ref(pack_file),
        )
        .await
    }

    async fn update_pack_file_many(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
        pack_files: &[PackFile],
    ) -> Result<(), InstanceError> {
        for (content_path, pack_file) in content_paths.iter().zip(pack_files) {
            let pack_file_path = self.get_pack_file_path(instance_id, content_path);
            let dto: PackFileV2 = pack_file.clone().into();
            write_toml_async(&pack_file_path, &dto)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        let mut pack = self.get_pack(instance_id).await?;
        let mut changed = false;

        for path in content_paths {
            if !pack.files.iter().any(|entry| &entry.file == path) {
                pack.files.push(PackEntry { file: path.clone() });
                changed = true;
            }
        }

        if changed {
            pack.files.sort_by(|a, b| a.file.cmp(&b.file));
            pack.files.dedup_by_key(|item| item.file.clone());

            self.update_pack(instance_id, &pack).await?;
        }

        Ok(())
    }

    async fn remove_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<(), InstanceError> {
        self.remove_pack_file_many(instance_id, std::slice::from_ref(content_path))
            .await
    }

    async fn remove_pack_file_many(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
    ) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        let mut success_deleted = Vec::new();

        for content_path in content_paths {
            let mut file_path = instance_dir.join(content_path.to_path_buf());

            if !file_path.exists() {
                file_path =
                    PathBuf::from(format!("{}{DISABLED_SUFFIX}", file_path.to_string_lossy()));
            }

            let _ = remove_file(file_path).await;

            let pack_file_path = self.get_pack_file_path(instance_id, content_path);

            if remove_file(&pack_file_path).await.is_ok() {
                success_deleted.push(content_path);
            }
        }

        let mut pack = self.get_pack(instance_id).await?;
        pack.files.retain(|e| !success_deleted.contains(&&e.file));
        pack.files.dedup_by_key(|item| item.file.clone());
        self.update_pack(instance_id, &pack).await
    }

    async fn remove_all_for_instance(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        let pack_dir = self.location_info.instance_pack_dir(instance_id);

        if pack_dir.exists() {
            remove_dir_all(&pack_dir)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    fn sha1_hash(c: char) -> crate::shared::hash::FileHash {
        crate::shared::hash::FileHash::new(
            crate::shared::hash::HashAlgorithm::Sha1,
            c.to_string().repeat(40),
        )
        .unwrap()
    }

    use super::*;

    #[tokio::test]
    async fn should_drop_an_index_entry_that_is_not_a_content_path_when_reading() {
        let temp = TempDir::new().unwrap();
        let storage = FsPackStorage::new(Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        )));
        let id = InstanceId::try_from("inst").unwrap();
        let good = ContentPath::parse("mods/a.jar").unwrap();
        storage
            .update_pack_file(
                &id,
                &good,
                &PackFile::from_hash("a.jar".into(), sha1_hash('a')),
            )
            .await
            .unwrap();
        tokio::fs::write(
            storage.get_pack_path(&id),
            "[[files]]\nfile = \"../x.jar\"\n\n[[files]]\nfile = \"mods/a.jar\"\n\n\
             [[files]]\nfile = \".metadata/instance.json\"\n",
        )
        .await
        .unwrap();

        let pack = storage.get_pack(&id).await.unwrap();
        let files = storage.get_pack_files(&id).await.unwrap();

        assert_eq!(pack.files.len(), 1);
        assert_eq!(pack.files[0].file, good);
        assert_eq!(files.len(), 1);
        assert_eq!(files[&good].hash, sha1_hash('a'));
    }

    #[tokio::test]
    async fn should_drop_an_index_entry_that_changes_when_normalized() {
        let temp = TempDir::new().unwrap();
        let storage = FsPackStorage::new(Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        )));
        let id = InstanceId::try_from("inst").unwrap();
        let good = ContentPath::parse("mods/a.jar").unwrap();
        storage
            .update_pack_file(
                &id,
                &good,
                &PackFile::from_hash("a.jar".into(), sha1_hash('a')),
            )
            .await
            .unwrap();
        tokio::fs::write(
            storage.get_pack_path(&id),
            r#"[[files]]
file = 'mods\a.jar'

[[files]]
file = "mods/a.jar"
"#,
        )
        .await
        .unwrap();

        let pack = storage.get_pack(&id).await.unwrap();

        assert_eq!(pack.files.len(), 1);
        assert_eq!(pack.files[0].file, good);
    }
}
