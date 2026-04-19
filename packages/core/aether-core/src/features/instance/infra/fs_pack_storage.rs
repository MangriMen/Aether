use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::{
    features::{
        instance::{InstanceError, Pack, PackEntry, PackFile, PackStorage, ProviderId},
        settings::LocationInfo,
    },
    shared::{ensure_read_toml_async, read_toml_async, remove_file, write_toml_async},
};

pub struct FsPackStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPackStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_pack_path(&self, instance_id: &str) -> PathBuf {
        self.location_info.instance_pack(instance_id)
    }

    fn get_pack_file_path(&self, instance_id: &str, content_path: &str) -> PathBuf {
        self.location_info
            .instance_pack_dir(instance_id)
            .join(content_path)
            .with_extension("toml")
    }
}

#[async_trait]
impl PackStorage for FsPackStorage {
    async fn get_pack(&self, instance_id: &str) -> Result<Pack, InstanceError> {
        Ok(ensure_read_toml_async(&self.get_pack_path(instance_id)).await?)
    }

    async fn update_pack(&self, instance_id: &str, pack: &Pack) -> Result<(), InstanceError> {
        write_toml_async(&self.get_pack_path(instance_id), &pack).await?;
        Ok(())
    }

    async fn get_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<PackFile, InstanceError> {
        Ok(read_toml_async(&self.get_pack_file_path(instance_id, content_path)).await?)
    }

    async fn find_by_provider_id(
        &self,
        instance_id: &str,
        provider_id: &ProviderId,
        content_id: &str,
    ) -> Result<Option<PackFile>, InstanceError> {
        let pack = self.get_pack(instance_id).await?;

        for entry in pack.files {
            if let Ok(pack_file) = self.get_pack_file(instance_id, &entry.file).await
                && let Some(update_map) = &pack_file.update
                && let Some(info) = update_map.get(provider_id)
                && info.content_id == content_id
            {
                return Ok(Some(pack_file));
            }
        }

        Ok(None)
    }

    async fn update_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        self.update_pack_file_many(
            instance_id,
            &[content_path.to_string()],
            std::slice::from_ref(pack_file),
        )
        .await
    }

    async fn update_pack_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
        pack_files: &[PackFile],
    ) -> Result<(), InstanceError> {
        for (content_path, pack_file) in content_paths.iter().zip(pack_files) {
            let pack_file_path = self.get_pack_file_path(instance_id, content_path);
            write_toml_async(&pack_file_path, &pack_file).await?;
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
        instance_id: &str,
        content_path: &str,
    ) -> Result<(), InstanceError> {
        self.remove_pack_file_many(instance_id, &[content_path.to_string()])
            .await
    }

    async fn remove_pack_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        let mut success_deleted = Vec::new();

        for content_path in content_paths {
            let mut file_path = instance_dir.join(content_path);

            if !file_path.exists() {
                file_path.set_extension("disabled");
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
}
