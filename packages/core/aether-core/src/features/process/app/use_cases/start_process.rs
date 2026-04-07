use std::sync::Arc;

use tokio::process::Command;

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, ProcessEventType},
        instance::InstanceStorage,
        process::{MinecraftProcessMetadata, ProcessError, ProcessStorage},
    },
    shared::IoError,
};

use super::{ManageProcessParams, ManageProcessUseCase};

pub struct StartProcessUseCase<E: EventEmitter, PS: ProcessStorage, IS: InstanceStorage> {
    event_emitter: Arc<E>,
    process_storage: Arc<PS>,
    manage_process_use_case: Arc<ManageProcessUseCase<E, PS, IS>>,
}

impl<E: EventEmitter + 'static, PS: ProcessStorage + 'static, IS: InstanceStorage + 'static>
    StartProcessUseCase<E, PS, IS>
{
    pub fn new(
        event_emitter: Arc<E>,
        process_storage: Arc<PS>,
        manage_process_use_case: Arc<ManageProcessUseCase<E, PS, IS>>,
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
                    process_uuid: process_uuid_clone,
                    instance_id: instance_id_clone,
                    post_exit_command,
                })
                .await;
        });

        self.event_emitter
            .emit_process_safe(
                instance_id.clone(),
                metadata.uuid(),
                "Launched Minecraft".to_string(),
                ProcessEventType::Launched,
            )
            .await;

        Ok(metadata)
    }
}
