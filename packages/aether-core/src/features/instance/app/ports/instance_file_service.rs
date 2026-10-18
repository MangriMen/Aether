use std::path::PathBuf;

use async_trait::async_trait;

use crate::features::instance::domain::{InstanceError, InstanceId};

#[async_trait]
pub trait InstanceFileService: Send + Sync {
    /// Create the instance directory for the given instance id.
    /// The directory path is derived from `LocationInfo`.
    async fn create_instance_dir(&self, instance_id: &InstanceId) -> Result<(), InstanceError>;

    /// Atomically create a unique instance directory inside the instances root.
    /// `base_name` should already be sanitised.
    /// Returns the unique folder name and the full path.
    async fn create_unique_instance_dir(
        &self,
        base_name: &str,
    ) -> Result<(String, PathBuf), InstanceError>;

    /// Remove the instance directory for the given instance id.
    async fn remove_instance_dir(&self, instance_id: &InstanceId) -> Result<(), InstanceError>;

    /// Write `bytes` to `relative_path` inside the instance directory, replacing whatever was
    /// there and creating the parent directories as needed.
    ///
    /// The single write path for plugins into an instance, so every write is resolved and
    /// bounded in one place. Anything that could leave the instance root (an absolute path, a
    /// `..` segment, a symlink pointing outside) is refused, as is a payload over the size limit.
    async fn write_instance_file(
        &self,
        instance_id: &InstanceId,
        relative_path: &str,
        bytes: &[u8],
    ) -> Result<(), InstanceError>;

    /// Read `relative_path` from inside the instance directory.
    ///
    /// Same path rules and the same size limit as [`InstanceFileService::write_instance_file`].
    async fn read_instance_file(
        &self,
        instance_id: &InstanceId,
        relative_path: &str,
    ) -> Result<Vec<u8>, InstanceError>;
}
