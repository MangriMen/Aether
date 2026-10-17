use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::RemoveContentUseCasePort;
use crate::features::{
    events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
    instance::{InstanceError, InstanceId, InstanceLocks, PackStorage, app::ContentFileService},
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

/// Removes content under the instance's lock ([`InstanceLocks`]): not while a plan is changing
/// the same files, which would bring a removed record back or leave a second copy of a mod.
pub struct RemoveContentUseCase {
    event_emitter: SharedEventEmitter,
    pack_storage: Arc<dyn PackStorage>,
    content_file_service: Arc<dyn ContentFileService>,
    instance_locks: Arc<InstanceLocks>,
}

impl RemoveContentUseCase {
    pub fn new(
        event_emitter: SharedEventEmitter,
        pack_storage: Arc<dyn PackStorage>,
        content_file_service: Arc<dyn ContentFileService>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            event_emitter,
            pack_storage,
            content_file_service,
            instance_locks,
        }
    }

    pub async fn execute(&self, input: RemoveContent) -> Result<(), InstanceError> {
        let RemoveContent {
            instance_id,
            content_paths,
        } = input;

        let id = InstanceId::try_from(instance_id.as_str())?;
        let _lock = self.instance_locks.lock(&id).await;

        // A record goes only with its file: one dropped while its file stays would leave the
        // file in the instance, untracked.
        let mut removed = Vec::with_capacity(content_paths.len());
        let mut first_error = None;
        for content_path in content_paths {
            match self
                .content_file_service
                .remove_content_files(&id, std::slice::from_ref(&content_path))
                .await
            {
                Ok(()) => removed.push(content_path),
                Err(err) => {
                    first_error.get_or_insert(err);
                }
            }
        }

        if !removed.is_empty() {
            self.pack_storage
                .remove_pack_file_many(&id, &removed)
                .await?;
        }

        self.event_emitter
            .emit_safe(InstanceEvent {
                event: InstanceEventType::Edited,
                instance_id: instance_id.clone(),
            })
            .await;

        first_error.map_or(Ok(()), Err)
    }
}

#[async_trait]
impl RemoveContentUseCasePort for RemoveContentUseCase {
    async fn execute(&self, input: RemoveContent) -> Result<(), InstanceError> {
        self.execute(input).await
    }
}
