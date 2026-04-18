use std::sync::Arc;

use tokio::process::Command;

use crate::{
    features::{
        events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
        instance::InstanceStorage,
        process::{MinecraftProcessMetadata, ProcessError, ProcessStorage},
    },
    shared::IoError,
};

use super::{ManageProcessParams, ManageProcessUseCase};

pub struct StartProcessUseCase<PS: ProcessStorage, IS: InstanceStorage> {
    event_emitter: SharedEventEmitter,
    process_storage: Arc<PS>,
    manage_process_use_case: Arc<ManageProcessUseCase<PS, IS>>,
}

impl<PS: ProcessStorage + 'static, IS: InstanceStorage + 'static> StartProcessUseCase<PS, IS> {
    pub fn new(
        event_emitter: SharedEventEmitter,
        process_storage: Arc<PS>,
        manage_process_use_case: Arc<ManageProcessUseCase<PS, IS>>,
    ) -> Self {
        Self {
            event_emitter,
            process_storage,
            manage_process_use_case,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        mut command: Command,
        post_exit_command: Option<String>,
    ) -> Result<MinecraftProcessMetadata, ProcessError> {
        let process = command.spawn().map_err(IoError::from)?;
        let metadata = MinecraftProcessMetadata::new(instance_id.clone());

        self.process_storage
            .insert(metadata.clone(), process)
            .await?;

        let manage_process_use_case = self.manage_process_use_case.clone();
        let instance_id_clone = instance_id.clone();
        let process_uuid_clone = metadata.uuid();

        tokio::spawn(async move {
            let _ = manage_process_use_case
                .execute(ManageProcessParams {
                    process_id: process_uuid_clone,
                    instance_id: instance_id_clone,
                    post_exit_command,
                })
                .await;
        });

        self.event_emitter
            .emit_safe(ProcessEvent {
                instance_id,
                process_id: metadata.uuid(),
                event: ProcessEventType::Launched,
                message: "Launched Minecraft".to_string(),
            })
            .await;

        Ok(metadata)
    }
}
