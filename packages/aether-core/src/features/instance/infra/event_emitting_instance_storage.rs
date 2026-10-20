use async_trait::async_trait;

use crate::features::{
    events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
    instance::{Instance, InstanceError, InstanceId, InstanceStorage},
};

pub struct EventEmittingInstanceStorage<IS> {
    event_emitter: SharedEventEmitter,
    instance_storage: IS,
}

impl<IS: InstanceStorage> EventEmittingInstanceStorage<IS> {
    pub fn new(event_emitter: SharedEventEmitter, instance_storage: IS) -> Self {
        Self {
            event_emitter,
            instance_storage,
        }
    }
}

#[async_trait]
impl<IS: InstanceStorage> InstanceStorage for EventEmittingInstanceStorage<IS> {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(self.instance_storage.list().await?)
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        Ok(self.instance_storage.get(id).await?)
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        self.instance_storage.upsert(instance).await?;

        self.event_emitter
            .emit_safe(InstanceEvent {
                event: InstanceEventType::Edited,
                instance_id: instance.id().clone(),
            })
            .await;

        Ok(())
    }

    async fn update(
        &self,
        id: &InstanceId,
        update_fn: &mut (dyn for<'i> FnMut(&'i mut Instance) -> Result<bool, InstanceError> + Send),
    ) -> Result<Instance, InstanceError> {
        let mut changed = false;
        let instance = self
            .instance_storage
            .update(id, &mut |instance| {
                changed = update_fn(instance)?;
                Ok(changed)
            })
            .await?;

        if changed {
            self.event_emitter
                .emit_safe(InstanceEvent {
                    event: InstanceEventType::Edited,
                    instance_id: id.clone(),
                })
                .await;
        }

        Ok(instance)
    }

    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError> {
        self.instance_storage.remove(id).await?;

        self.event_emitter
            .emit_safe(InstanceEvent {
                event: InstanceEventType::Removed,
                instance_id: id.clone(),
            })
            .await;

        Ok(())
    }
}
