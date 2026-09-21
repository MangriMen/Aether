use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use tokio::fs;

use crate::{
    features::{
        instance::{app::InstanceFileService, domain::InstanceError},
        settings::LocationInfo,
    },
    shared::io::infra::{create_dir_all, remove_dir_all},
};

use super::instance_relative_path::{check_size, resolve_instance_file_path};

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

    async fn create_unique_instance_dir(
        &self,
        base_name: &str,
    ) -> Result<(String, PathBuf), InstanceError> {
        let instances_root = self.location_info.instances_dir();

        // Ensure the instances root directory exists first
        create_dir_all(&instances_root)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;

        let mut counter = 0u32;
        loop {
            let name = if counter == 0 {
                base_name.to_owned()
            } else {
                format!("{base_name}-{counter}")
            };
            let path = instances_root.join(&name);
            match fs::create_dir(&path).await {
                Ok(()) => return Ok((name, path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    counter += 1;
                }
                Err(e) => {
                    return Err(InstanceError::Storage(format!(
                        "Failed to create instance directory '{}': {e}",
                        path.display()
                    )));
                }
            }
        }
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

    async fn write_instance_file(
        &self,
        instance_id: &str,
        relative_path: &str,
        bytes: &[u8],
    ) -> Result<(), InstanceError> {
        check_size(relative_path, bytes.len() as u64)?;

        let instance_dir = self.location_info.instance_dir(instance_id);
        let target = resolve_instance_file_path(instance_id, &instance_dir, relative_path)?;

        if let Some(parent) = target.parent() {
            create_dir_all(parent)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        // Resolve a second time, now that the parent chain exists. `create_dir_all` only ever
        // creates plain directories, so this cannot newly pass — but it can newly fail, if
        // something replaced a component with a symlink since the first check.
        let target = resolve_instance_file_path(instance_id, &instance_dir, relative_path)?;

        fs::write(&target, bytes).await.map_err(|err| {
            InstanceError::Storage(format!("Failed to write \"{}\": {err}", target.display()))
        })
    }

    async fn read_instance_file(
        &self,
        instance_id: &str,
        relative_path: &str,
    ) -> Result<Vec<u8>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        let target = resolve_instance_file_path(instance_id, &instance_dir, relative_path)?;

        let metadata = fs::metadata(&target).await.map_err(|err| {
            InstanceError::Storage(format!("Failed to read \"{}\": {err}", target.display()))
        })?;

        if !metadata.is_file() {
            return Err(InstanceError::InvalidRelativePath {
                path: relative_path.to_owned(),
                reason: "path is not a regular file".to_owned(),
            });
        }

        // Checked from the metadata rather than after reading, so an oversized file is never
        // pulled into memory at all.
        check_size(relative_path, metadata.len())?;

        fs::read(&target).await.map_err(|err| {
            InstanceError::Storage(format!("Failed to read \"{}\": {err}", target.display()))
        })
    }
}
