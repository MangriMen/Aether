use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::{app::InstanceFileService, domain::InstanceError},
        settings::LocationInfo,
    },
    shared::io::infra::{create_dir_all, remove_dir_all},
};

pub struct FsInstanceFileService {
    location_info: Arc<LocationInfo>,
}

impl FsInstanceFileService {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }
}

#[async_trait]
impl InstanceFileService for FsInstanceFileService {
    async fn create_instance_dir(&self, instance_id: &str) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        create_dir_all(&instance_dir)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }

    async fn remove_instance_dir(&self, instance_id: &str) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        if !instance_dir.exists() {
            return Ok(());
        }

        remove_dir_all(&instance_dir)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }
}
