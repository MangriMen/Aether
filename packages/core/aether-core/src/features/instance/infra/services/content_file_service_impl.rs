use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::{app::ContentFileService, domain::InstanceError},
        settings::LocationInfo,
    },
    shared::io::infra::{create_dir_all, remove_file, rename},
};

pub struct FsContentFileService {
    location_info: Arc<LocationInfo>,
}

impl FsContentFileService {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }
}

#[async_trait]
impl ContentFileService for FsContentFileService {
    async fn remove_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        for content_path in content_paths {
            let mut file_path = instance_dir.join(content_path);

            if !file_path.exists() {
                // TODO rework disable logic to make it more transparent
                file_path = PathBuf::from(format!("{}.disabled", file_path.to_string_lossy()));
            }

            let _ = remove_file(file_path).await;
        }

        Ok(())
    }

    async fn enable_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        for content_path in content_paths {
            let enabled_path = instance_dir.join(content_path);

            if enabled_path.exists() {
                continue;
            }

            let disabled_path = instance_dir.join(format!("{content_path}.disabled"));

            if !disabled_path.exists() {
                continue;
            }

            rename(disabled_path, enabled_path)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    async fn disable_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        for content_path in content_paths {
            let disabled_path = instance_dir.join(format!("{content_path}.disabled"));

            if disabled_path.exists() {
                continue;
            }

            let enabled_path = instance_dir.join(content_path);

            if !enabled_path.exists() {
                continue;
            }

            rename(enabled_path, disabled_path)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    async fn install_content_file(
        &self,
        instance_id: &str,
        content_path: &str,
        temp_path: &Path,
    ) -> Result<(), InstanceError> {
        let absolute_content_path = self
            .location_info
            .instance_dir(instance_id)
            .join(content_path);

        if let Some(parent) = absolute_content_path.parent() {
            create_dir_all(parent)
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;
        }

        rename(temp_path, absolute_content_path)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }
}
