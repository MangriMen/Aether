use async_trait::async_trait;

use crate::features::instance::{InstanceError, PlatformId, PlatformProviderSelection, ProviderId};

/// No provider chosen for any platform, so the rest of the rule decides: the only provider, else
/// the built-in one.
pub struct NoPlatformProviderSelection;

#[async_trait]
impl PlatformProviderSelection for NoPlatformProviderSelection {
    async fn selected_provider(
        &self,
        _platform: &PlatformId,
    ) -> Result<Option<ProviderId>, InstanceError> {
        Ok(None)
    }
}
