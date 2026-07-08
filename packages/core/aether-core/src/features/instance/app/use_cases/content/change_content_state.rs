use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::ChangeContentStateUseCasePort;
use crate::features::{
    events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
    instance::{InstanceError, app::ContentFileService},
};

pub enum ContentStateAction {
    Enable,
    Disable,
}

pub struct ChangeContentState {
    pub instance_id: String,
    pub content_paths: Vec<String>,
    pub action: ContentStateAction,
}

impl ChangeContentState {
    pub fn single(instance_id: String, content_path: String, action: ContentStateAction) -> Self {
        Self {
            instance_id,
            content_paths: vec![content_path],
            action,
        }
    }

    pub fn multiple(
        instance_id: String,
        content_paths: Vec<String>,
        action: ContentStateAction,
    ) -> Self {
        Self {
            instance_id,
            content_paths,
            action,
        }
    }
}

pub struct ChangeContentStateUseCase {
    event_emitter: SharedEventEmitter,
    content_file_service: Arc<dyn ContentFileService>,
}

impl ChangeContentStateUseCase {
    pub fn new(
        event_emitter: SharedEventEmitter,
        content_file_service: Arc<dyn ContentFileService>,
    ) -> Self {
        Self {
            event_emitter,
            content_file_service,
        }
    }

    pub async fn execute(&self, input: ChangeContentState) -> Result<(), InstanceError> {
        let ChangeContentState {
            instance_id,
            content_paths,
            action,
        } = input;

        match action {
            ContentStateAction::Enable => {
                self.content_file_service
                    .enable_content_files(&instance_id, content_paths.as_slice())
                    .await?;
            }
            ContentStateAction::Disable => {
                self.content_file_service
                    .disable_content_files(&instance_id, content_paths.as_slice())
                    .await?;
            }
        }

        self.event_emitter
            .emit_safe(InstanceEvent {
                event: InstanceEventType::Edited,
                instance_id,
            })
            .await;

        Ok(())
    }
}

#[async_trait]
impl ChangeContentStateUseCasePort for ChangeContentStateUseCase {
    async fn execute(&self, input: ChangeContentState) -> Result<(), InstanceError> {
        self.execute(input).await
    }
}
