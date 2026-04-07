use std::sync::Arc;

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, InstanceEventType},
        instance::InstanceError,
        settings::LocationInfo,
    },
    shared::rename,
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

pub struct ChangeContentStateUseCase<E: EventEmitter> {
    event_emitter: Arc<E>,
    location_info: Arc<LocationInfo>,
}

impl<E: EventEmitter> ChangeContentStateUseCase<E> {
    pub fn new(event_emitter: Arc<E>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            event_emitter,
            location_info,
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
                self.enable_many(&instance_id, content_paths.as_slice())
                    .await
            }
            ContentStateAction::Disable => {
                self.disable_many(&instance_id, content_paths.as_slice())
                    .await
            }
        }?;

        Ok(())
    }

    pub async fn enable_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        for content_path in content_paths {
            self.enable(instance_id, content_path).await?;
        }

        self.event_emitter
            .emit_instance_safe(instance_id.to_string(), InstanceEventType::Edited)
            .await;

        Ok(())
    }

    pub async fn disable_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        for content_path in content_paths {
            self.disable(instance_id, content_path).await?;
        }

        self.event_emitter
            .emit_instance_safe(instance_id.to_string(), InstanceEventType::Edited)
            .await;

        Ok(())
    }

    async fn enable(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<Option<String>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        let absolute_enabled_content_path = instance_dir.join(content_path);

        if absolute_enabled_content_path.exists() {
            return Ok(None);
        }

        let disabled_content_path = format!("{content_path}.disabled");
        let absolute_disabled_content_path = instance_dir.join(disabled_content_path);

        if !absolute_disabled_content_path.exists() {
            return Ok(None);
        }

        rename(
            absolute_disabled_content_path,
            absolute_enabled_content_path,
        )
        .await?;

        Ok(Some(content_path.to_string()))
    }

    async fn disable(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<Option<String>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        let disabled_content_path = format!("{content_path}.disabled");
        let absolute_disabled_content_path = instance_dir.join(disabled_content_path.clone());

        if absolute_disabled_content_path.exists() {
            return Ok(None);
        }

        let absolute_enabled_content_path = instance_dir.join(content_path);

        if !absolute_enabled_content_path.exists() {
            return Ok(None);
        }

        rename(
            absolute_enabled_content_path,
            absolute_disabled_content_path,
        )
        .await?;

        Ok(Some(disabled_content_path))
    }
}
