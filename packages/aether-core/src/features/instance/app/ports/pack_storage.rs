use std::collections::HashMap;

use async_trait::async_trait;

use crate::features::instance::domain::{
    ContentId, ContentPath, InstanceError, InstanceId, Pack, PackFile, ProviderId,
};

#[async_trait]
pub trait PackStorage: Send + Sync {
    async fn get_pack(&self, instance_id: &InstanceId) -> Result<Pack, InstanceError>;

    async fn update_pack(&self, instance_id: &InstanceId, pack: &Pack)
    -> Result<(), InstanceError>;

    async fn get_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<PackFile, InstanceError>;

    /// The record of a file, or `None` when there is no usable one: its stored hash is invalid.
    /// Such a record is as good as none: the caller hashes the file again and writes a new one.
    async fn find_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<Option<PackFile>, InstanceError> {
        self.get_pack_file(instance_id, content_path)
            .await
            .map(Some)
    }

    /// Every file recorded for the instance, by content path — what an install plan is diffed
    /// against. The default asks for each file of [`Self::get_pack`] in turn.
    async fn get_pack_files(
        &self,
        instance_id: &InstanceId,
    ) -> Result<HashMap<ContentPath, PackFile>, InstanceError> {
        let mut files = HashMap::new();

        for entry in self.get_pack(instance_id).await?.files {
            let pack_file = self.get_pack_file(instance_id, &entry.file).await?;
            files.insert(entry.file, pack_file);
        }

        Ok(files)
    }

    async fn find_by_provider_id(
        &self,
        instance_id: &InstanceId,
        provider_id: &ProviderId,
        content_id: &ContentId,
    ) -> Result<Option<PackFile>, InstanceError>;

    async fn update_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError>;

    async fn update_pack_file_many(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
        pack_files: &[PackFile],
    ) -> Result<(), InstanceError>;

    async fn remove_pack_file(
        &self,
        instance_id: &InstanceId,
        content_path: &ContentPath,
    ) -> Result<(), InstanceError>;

    async fn remove_pack_file_many(
        &self,
        instance_id: &InstanceId,
        content_paths: &[ContentPath],
    ) -> Result<(), InstanceError>;

    async fn remove_all_for_instance(&self, instance_id: &InstanceId) -> Result<(), InstanceError>;
}
