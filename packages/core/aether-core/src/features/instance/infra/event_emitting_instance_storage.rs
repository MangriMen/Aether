use std::sync::Arc;

use async_trait::async_trait;
use log::error;

use crate::features::{
    events::{EventEmitter, EventEmitterExt, InstanceEventType},
    instance::{Instance, InstanceError, InstanceStorage},
};

pub struct EventEmittingInstanceStorage<E, IS> {
    event_emitter: Arc<E>,
    instance_storage: IS,
}

impl<E: EventEmitter, IS: InstanceStorage> EventEmittingInstanceStorage<E, IS> {
    pub fn new(event_emitter: Arc<E>, instance_storage: IS) -> Self {
        Self {
            event_emitter,
            instance_storage,
        }
    }
}

#[async_trait]
impl<E: EventEmitter, IS: InstanceStorage> InstanceStorage for EventEmittingInstanceStorage<E, IS> {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(self.instance_storage.list().await?)
    }

    async fn get(&self, id: &str) -> Result<Instance, InstanceError> {
        Ok(self.instance_storage.get(id).await?)
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        self.instance_storage.upsert(instance).await?;
        if let Err(e) = self
            .event_emitter
            .emit_instance(instance.id.to_string(), InstanceEventType::Edited)
            .await
        {
            error!("Failed to emit event: {}", e);
        }

        Ok(())
    }

    async fn remove(&self, id: &str) -> Result<(), InstanceError> {
        self.instance_storage.remove(id).await?;
        if let Err(e) = self
            .event_emitter
            .emit_instance(id.to_string(), InstanceEventType::Removed)
            .await
        {
            error!("Failed to emit event: {}", e);
        }

        Ok(())
    }
}
