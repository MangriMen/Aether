use async_trait::async_trait;

use crate::shared::{CapabilityEntry, RegistryError};

#[async_trait]
pub trait CapabilityRegistry<C: Send + Sync + Clone>: Send + Sync {
    fn get_type(&self) -> &'static str;

    async fn add(
        &self,
        plugin_id: String,
        capability_id: String,
        capability: C,
    ) -> Result<(), RegistryError>;

    async fn list(&self) -> Result<Vec<CapabilityEntry<C>>, RegistryError>;

    async fn find_by_capability_id(
        &self,
        capability_id: &str,
    ) -> Result<Vec<CapabilityEntry<C>>, RegistryError>;

    async fn find_by_plugin_and_capability_id(
        &self,
        plugin_id: &str,
        capability_id: &str,
    ) -> Result<CapabilityEntry<C>, RegistryError>;

    async fn remove(&self, plugin_id: String, capability_id: String) -> Result<(), RegistryError>;
}
