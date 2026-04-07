use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::{Instance, InstanceError, InstanceStorage},
        settings::LocationInfo,
    },
    shared::{read_json_async, remove_dir_all, write_json_async},
};

pub struct FsInstanceStorage {
    location_info: Arc<LocationInfo>,
}

impl FsInstanceStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
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
            let instance = read_json_async::<Instance>(&instance_json_path).await;

            match instance {
                Ok(instance) => instances.push(instance),
                Err(err) => {
                    tracing::debug!("Failed to read instance {:?}", err)
                }
            }
        }

        return Ok(instances);
    }

    async fn get(&self, id: &str) -> Result<Instance, InstanceError> {
        Ok(read_json_async(&self.location_info.instance_metadata_file(id)).await?)
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let path = self.location_info.instance_metadata_file(&instance.id);
        write_json_async(&path, instance).await?;
        Ok(())
    }

    async fn remove(&self, id: &str) -> Result<(), InstanceError> {
        let path = self.location_info.instance_dir(id);
        remove_dir_all(&path).await?;
        Ok(())
    }
}
