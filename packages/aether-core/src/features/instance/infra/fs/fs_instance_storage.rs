use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use tracing::warn;

use crate::{
    features::{
        instance::{
            Instance, InstanceChange, InstanceError, InstanceId, InstanceStorage,
            infra::fs::instance::InstanceV2,
        },
        settings::LocationInfo,
    },
    shared::io::infra::{read_json_async, remove_dir_all, write_json_async},
};

pub struct FsInstanceStorage {
    location_info: Arc<LocationInfo>,
    // Serializes read-modify-write cycles so concurrent edits keep each other's fields.
    write_lock: tokio::sync::Mutex<()>,
}

impl FsInstanceStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self {
            location_info,
            write_lock: tokio::sync::Mutex::new(()),
        }
    }

    async fn write(&self, instance: &Instance) -> Result<(), InstanceError> {
        let path = self.location_info.instance_metadata_file(instance.id());
        write_json_async(&path, InstanceV2::from(instance))
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        Ok(())
    }

    async fn load_instance_internal(&self, path: &Path) -> Result<Instance, InstanceError> {
        match read_json_async::<InstanceV2>(path).await {
            Ok(instance) => Instance::try_from(instance),
            Err(original_err) => match read_json_async::<super::instance::InstanceV1>(path).await {
                Ok(v1) => {
                    warn!("Migrating instance from V1 at {:?}", path);
                    let migrated = Instance::try_from(v1)?;
                    let v2 = InstanceV2::from(&migrated);

                    if let Err(e) = write_json_async(path, &v2).await {
                        warn!("Failed to save migrated instance: {:?}", e);
                    }

                    Ok(migrated)
                }
                Err(_) => Err(InstanceError::Storage(original_err.to_string())),
            },
        }
    }
}

#[async_trait]
impl InstanceStorage for FsInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        let instances_dir = self.location_info.instances_dir();

        if !instances_dir.exists() {
            return Ok(Vec::default());
        }

        let Ok(entries) = instances_dir.read_dir() else {
            return Ok(Vec::default());
        };

        let mut instances = Vec::new();

        for entry in entries.flatten() {
            let instance_json_path = self
                .location_info
                .instance_metadata_file_with_instance_dir(&entry.path());

            match self.load_instance_internal(&instance_json_path).await {
                Ok(instance) => instances.push(instance),
                Err(err) => {
                    warn!(
                        "Failed to read instance at {:?}: {:?}",
                        instance_json_path, err
                    );
                }
            }
        }

        return Ok(instances);
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        let path = self.location_info.instance_metadata_file(id);
        self.load_instance_internal(&path).await
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let _guard = self.write_lock.lock().await;
        self.write(instance).await
    }

    async fn update(
        &self,
        id: &InstanceId,
        change: InstanceChange<'_>,
    ) -> Result<Instance, InstanceError> {
        let _guard = self.write_lock.lock().await;
        let mut instance = self.get(id).await?;
        if change(&mut instance)? {
            self.write(&instance).await?;
        }
        Ok(instance)
    }

    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError> {
        let _guard = self.write_lock.lock().await;
        let path = self.location_info.instance_dir(id);
        remove_dir_all(&path)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use super::*;
    use crate::features::{instance::InstanceBuilder, minecraft::ModLoader};

    async fn store(storage: &FsInstanceStorage, id: &InstanceId) {
        let instance = InstanceBuilder::new(
            id.clone(),
            "Stored".to_owned(),
            "1.20.1".to_owned(),
            ModLoader::Vanilla,
        )
        .build();
        storage.upsert(&instance).await.unwrap();
    }

    /// Stores `kept` as usual and `broken` with an id in its file that leaves the instances
    /// directory, as a plugin could have put there.
    async fn setup_broken() -> (TempDir, FsInstanceStorage, InstanceId, InstanceId) {
        let temp_dir = TempDir::new().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            temp_dir.path().join("settings"),
            temp_dir.path().to_path_buf(),
        ));
        let storage = FsInstanceStorage::new(location_info.clone());
        let kept = InstanceId::try_from("kept").unwrap();
        let broken = InstanceId::try_from("broken").unwrap();
        store(&storage, &kept).await;
        store(&storage, &broken).await;

        let path = location_info.instance_metadata_file(&broken);
        let mut json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        json["id"] = "../plugins/planted".into();
        std::fs::write(&path, json.to_string()).unwrap();

        (temp_dir, storage, kept, broken)
    }

    #[tokio::test]
    async fn should_not_find_an_instance_when_the_id_in_its_file_leaves_the_instances_directory() {
        let (_temp_dir, storage, _kept, broken) = setup_broken().await;

        let err = storage.get(&broken).await.unwrap_err();

        assert!(matches!(err, InstanceError::NotFound { .. }), "{err}");
    }

    #[tokio::test]
    async fn should_skip_an_instance_when_the_id_in_its_file_is_invalid() {
        let (_temp_dir, storage, kept, _broken) = setup_broken().await;

        let instances = storage.list().await.unwrap();

        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id(), &kept);
    }
}
