use async_trait::async_trait;

use crate::features::instance::domain::{ImporterCapabilityMetadata, InstanceError, InstanceId};

#[async_trait]
pub trait Importer: Send + Sync {
    fn metadata(&self) -> &ImporterCapabilityMetadata;

    /// Import the instance at `path`. Gives back the instances created for a pack along the
    /// way, still `PackInstalling`: the caller makes them launchable.
    async fn import(&self, path: &str) -> Result<Vec<InstanceId>, InstanceError>;
}
