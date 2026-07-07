use std::sync::Arc;

use uuid::Uuid;

use crate::{
    features::{
        events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
        process::{ManageProcessService, ProcessError, ProcessStorage, TrackProcessService},
        settings::LocationInfo,
    },
    shared::{io::domain::IoError, serializable_command::domain::SerializableCommand},
};

use super::TrackProcessParams;

pub struct ManageProcessParams {
    pub process_id: Uuid,
    pub instance_id: String,
    pub post_exit_command: String,
}

pub struct ManageProcessUseCase {
    event_emitter: SharedEventEmitter,
    process_storage: Arc<dyn ProcessStorage>,
    track_process_service: Arc<dyn TrackProcessService>,
    location_info: Arc<LocationInfo>,
}

#[async_trait::async_trait]
impl ManageProcessService for ManageProcessUseCase {
    async fn execute(&self, params: ManageProcessParams) -> Result<(), ProcessError> {
        let ManageProcessParams {
            process_id,
            instance_id,
            post_exit_command,
        } = params;

        let mc_exit_status = self
            .track_process_service
            .execute(TrackProcessParams {
                process_id,
                instance_id: instance_id.clone(),
            })
            .await;

        self.process_storage.remove(process_id).await?;

        self.event_emitter
            .emit_safe(ProcessEvent {
                instance_id: instance_id.clone(),
                process_id,
                event: ProcessEventType::Finished,
                message: "Exited process".to_string(),
            })
            .await;

        if mc_exit_status.success() && !post_exit_command.is_empty() {
            self.run_post_exit(&post_exit_command, &instance_id)?;
        }

        Ok(())
    }
}

impl ManageProcessUseCase {
    pub fn new(
        event_emitter: SharedEventEmitter,
        process_storage: Arc<dyn ProcessStorage>,
        track_process_service: Arc<dyn TrackProcessService>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            event_emitter,
            process_storage,
            track_process_service,
            location_info,
        }
    }

    fn run_post_exit(&self, command: &str, instance_id: &str) -> Result<(), ProcessError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        if let Ok(cmd) = SerializableCommand::from_string(command, Some(&instance_dir)) {
            cmd.to_tokio_command()
                .spawn()
                .map_err(|e| IoError::with_path(e, instance_dir))?;
        }
        Ok(())
    }
}
