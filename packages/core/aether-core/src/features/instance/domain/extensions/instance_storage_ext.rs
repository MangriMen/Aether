use crate::features::instance::{Instance, InstanceError, InstanceStorage};
use async_trait::async_trait;

#[async_trait]
pub trait InstanceStorageExt: InstanceStorage {
    async fn upsert_with<F>(&self, id: &str, update_fn: F) -> Result<(), InstanceError>
    where
        F: FnOnce(&mut Instance) -> Result<(), InstanceError> + Send,
    {
        let mut instance = self.get(id).await?;
        update_fn(&mut instance)?;
        self.upsert(&instance).await
    }
}

#[async_trait]
impl<IS: InstanceStorage> InstanceStorageExt for IS {}
