use std::path::Path;

use async_trait::async_trait;

use crate::features::instance::domain::InstanceError;

#[async_trait]
pub trait ContentFileService: Send + Sync {
    /// Removes content files from disk for the given instance.
    /// Also handles `.disabled` variants.
    async fn remove_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;

    /// Renames `.disabled` → original for the given content paths.
    async fn enable_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;

    /// Renames original → `.disabled` for the given content paths.
    async fn disable_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;

    /// Creates parent directories and moves a temp file to the final
    /// content path inside the instance directory.
    async fn install_content_file(
        &self,
        instance_id: &str,
        content_path: &str,
        temp_path: &Path,
    ) -> Result<(), InstanceError>;
}
