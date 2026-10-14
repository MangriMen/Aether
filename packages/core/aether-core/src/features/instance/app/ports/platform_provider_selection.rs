use async_trait::async_trait;

use crate::features::instance::domain::{InstanceError, ProviderId};

/// Which provider the user chose for a platform: the map "platform → [`ProviderId`]" from the
/// settings.
///
/// Only the choice is stored here; whether the chosen provider is available, and what happens
/// without a choice, is the executor's rule.
#[async_trait]
pub trait PlatformProviderSelection: Send + Sync {
    /// The provider chosen for `platform`, if one is.
    async fn selected_provider(&self, platform: &str) -> Result<Option<ProviderId>, InstanceError>;
}
