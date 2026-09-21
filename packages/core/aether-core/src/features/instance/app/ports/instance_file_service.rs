use std::path::PathBuf;

use async_trait::async_trait;

use crate::features::instance::domain::InstanceError;

#[async_trait]
pub trait InstanceFileService: Send + Sync {
    /// Create the instance directory for the given instance id.
    /// The directory path is derived from `LocationInfo`.
    async fn create_instance_dir(&self, instance_id: &str) -> Result<(), InstanceError>;

    /// Atomically create a unique instance directory inside the instances root.
    /// `base_name` should already be sanitised.
    /// Returns the unique folder name and the full path.
    async fn create_unique_instance_dir(
        &self,
        base_name: &str,
    ) -> Result<(String, PathBuf), InstanceError>;

    /// Remove the instance directory for the given instance id.
    async fn remove_instance_dir(&self, instance_id: &str) -> Result<(), InstanceError>;

    /// Write `bytes` to `relative_path` inside the instance directory, replacing whatever was
    /// there and creating the parent directories as needed (T-1.3).
    ///
    /// This is the single write path into an instance: plugins reach it through the
    /// `write_instance_file` host function rather than through their own WASI mount, so that
    /// every write is resolved and bounded in one place. `relative_path` is a `/`-separated
    /// chain of plain name segments; anything that could leave the instance root — an absolute
    /// path, a `..` segment, a symlink pointing outside — is refused, as is a payload over the
    /// size limit.
    async fn write_instance_file(
        &self,
        instance_id: &str,
        relative_path: &str,
        bytes: &[u8],
    ) -> Result<(), InstanceError>;

    /// Read `relative_path` from inside the instance directory (T-1.3).
    ///
    /// Same path rules and the same size limit as [`InstanceFileService::write_instance_file`].
    async fn read_instance_file(
        &self,
        instance_id: &str,
        relative_path: &str,
    ) -> Result<Vec<u8>, InstanceError>;
}
