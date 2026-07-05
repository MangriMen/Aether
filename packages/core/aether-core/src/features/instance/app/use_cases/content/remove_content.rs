use std::sync::Arc;

use crate::features::{
    events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
    instance::{InstanceError, PackStorage, app::ContentFileService},
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

pub struct RemoveContentUseCase {
    event_emitter: SharedEventEmitter,
    pack_storage: Arc<dyn PackStorage>,
    content_file_service: Arc<dyn ContentFileService>,
}

impl RemoveContentUseCase {
    pub fn new(
        event_emitter: SharedEventEmitter,
        pack_storage: Arc<dyn PackStorage>,
        content_file_service: Arc<dyn ContentFileService>,
    ) -> Self {
        Self {
            event_emitter,
            pack_storage,
            content_file_service,
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

        self.content_file_service
            .remove_content_files(&instance_id, content_paths.as_slice())
            .await?;

        self.event_emitter
            .emit_safe(InstanceEvent {
                event: InstanceEventType::Edited,
                instance_id: instance_id.clone(),
            })
            .await;

        Ok(())
    }
}
