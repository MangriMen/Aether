use async_trait::async_trait;
use dashmap::DashMap;

use crate::shared::{CapabilityEntry, CapabilityRegistry, RegistryError};

type CapabilityKey = (String, String); // (plugin_id, capability.id)

pub struct MemoryCapabilityRegistry<C>
where
    C: Send + Sync + Clone,
{
    capability_type: &'static str, // Default, usually overridden
    capabilities: DashMap<CapabilityKey, CapabilityEntry<C>>,
}

impl<C> MemoryCapabilityRegistry<C>
where
    C: Send + Sync + Clone,
{
    pub fn new(capability_type: &'static str) -> Self {
        Self {
            capability_type,
            capabilities: DashMap::new(),
        }
    }
}

impl<C> Default for MemoryCapabilityRegistry<C>
where
    C: Send + Sync + Clone,
{
    fn default() -> Self {
        Self {
            capability_type: "unknown",
            capabilities: DashMap::new(),
        }
    }
}

#[async_trait]
impl<C> CapabilityRegistry<C> for MemoryCapabilityRegistry<C>
where
    C: Send + Sync + Clone,
{
    fn get_type(&self) -> &'static str {
        self.capability_type
    }

    async fn add(
        &self,
        plugin_id: String,
        capability_id: String,
        capability: C,
    ) -> Result<(), RegistryError> {
        let key = (plugin_id.clone(), capability_id);
        self.capabilities.insert(
            key,
            CapabilityEntry {
                plugin_id,
                capability,
            },
        );
        Ok(())
    }

    async fn list(&self) -> Result<Vec<CapabilityEntry<C>>, RegistryError> {
        Ok(self
            .capabilities
            .iter()
            .map(|item| item.value().clone())
            .collect())
    }

    async fn find_by_capability_id(
        &self,
        capability_id: &str,
    ) -> Result<Vec<CapabilityEntry<C>>, RegistryError> {
        let capabilities: Vec<CapabilityEntry<C>> = self
            .capabilities
            .iter()
            .filter(|item| item.key().1 == capability_id)
            .map(|item| item.value().clone())
            .collect();

        if capabilities.is_empty() {
            return Err(RegistryError::CapabilityNotFound {
                capability_type: self.capability_type,
                capability_id: capability_id.to_owned(),
            });
        }

        Ok(capabilities)
    }

    async fn find_by_plugin_and_capability_id(
        &self,
        plugin_id: &str,
        capability_id: &str,
    ) -> Result<CapabilityEntry<C>, RegistryError> {
        let key = (plugin_id.to_owned(), capability_id.to_owned());

        let entry = self
            .capabilities
            .get(&key)
            .ok_or(RegistryError::CapabilityNotFound {
                capability_type: self.capability_type,
                capability_id: capability_id.to_owned(),
            })?;

        Ok(entry.clone())
    }

    async fn remove(&self, plugin_id: String, capability_id: String) -> Result<(), RegistryError> {
        let key = (plugin_id, capability_id);
        self.capabilities.remove(&key);
        Ok(())
    }
}
