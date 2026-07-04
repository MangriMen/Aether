use async_trait::async_trait;

use crate::features::instance::domain::InstanceError;

#[async_trait]
pub trait ContentFileService: Send + Sync {
    async fn remove_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;

    async fn enable_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;

    async fn disable_content_files(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;
}
