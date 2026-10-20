use async_trait::async_trait;

use crate::features::instance::domain::{Instance, InstanceError, InstanceId};

#[async_trait]
pub trait InstanceStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError>;
    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError>;
    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError>;
    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError>;

    /// Read-modify-write of one instance. `update_fn` returns whether it changed
    /// anything; nothing is written otherwise. Implementations should make the
    /// whole cycle atomic so concurrent edits do not overwrite each other's fields.
    async fn update(
        &self,
        id: &InstanceId,
        update_fn: &mut (dyn for<'i> FnMut(&'i mut Instance) -> Result<bool, InstanceError> + Send),
    ) -> Result<Instance, InstanceError> {
        let mut instance = self.get(id).await?;
        if update_fn(&mut instance)? {
            self.upsert(&instance).await?;
        }
        Ok(instance)
    }
}
