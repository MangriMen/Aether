use async_trait::async_trait;

use crate::features::instance::app::InstanceStorage;
use crate::features::instance::domain::{Instance, InstanceError, InstanceId};

#[async_trait]
pub trait InstanceStorageExt: InstanceStorage {
    async fn upsert_with<F>(&self, id: &InstanceId, update_fn: F) -> Result<(), InstanceError>
    where
        F: FnOnce(&mut Instance) -> Result<(), InstanceError> + Send,
    {
        self.update(
            id,
            Box::new(|instance| {
                update_fn(instance)?;
                Ok(true)
            }),
        )
        .await
        .map(|_| ())
    }
}

#[async_trait]
impl<IS: ?Sized + InstanceStorage> InstanceStorageExt for IS {}
