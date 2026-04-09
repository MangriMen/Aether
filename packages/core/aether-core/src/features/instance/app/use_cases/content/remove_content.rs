use std::sync::Arc;

use crate::features::{
    events::{EventEmitterExt, InstanceEventType, SharedEventEmitter},
    instance::{InstanceError, PackStorage},
};

pub struct RemoveContent {
    instance_id: String,
    content_paths: Vec<String>,
}

impl RemoveContent {
    pub fn multiple(instance_id: String, content_paths: Vec<String>) -> Self {
        Self {
            instance_id,
            content_paths,
        }
    }
}

pub struct RemoveContentUseCase<PS: PackStorage> {
    event_emitter: SharedEventEmitter,
    pack_storage: Arc<PS>,
}

impl<PS: PackStorage> RemoveContentUseCase<PS> {
    pub fn new(event_emitter: SharedEventEmitter, pack_storage: Arc<PS>) -> Self {
        Self {
            event_emitter,
            pack_storage,
        }
    }

    pub async fn execute(&self, input: RemoveContent) -> Result<(), InstanceError> {
        let RemoveContent {
            instance_id,
            content_paths,
        } = input;

        self.pack_storage
            .remove_pack_file_many(&instance_id, content_paths.as_slice())
            .await?;

        self.event_emitter
            .emit_instance_safe(instance_id.to_string(), InstanceEventType::Edited)
            .await;

        Ok(())
    }
}
