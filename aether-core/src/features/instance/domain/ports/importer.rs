use async_trait::async_trait;

use crate::features::instance::{ImporterCapabilityMetadata, InstanceError};

#[async_trait]
pub trait Importer: Send + Sync {
    fn metadata(&self) -> &ImporterCapabilityMetadata;

    async fn import(&self, path: &str) -> Result<(), InstanceError>;
}
