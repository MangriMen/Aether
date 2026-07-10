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
}
