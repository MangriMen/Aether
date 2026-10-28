use std::{process::ExitStatus, sync::Arc};

use uuid::Uuid;

use crate::{
    features::{
        events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
        instance::InstanceId,
        process::{ManageProcessService, ProcessError, ProcessStorage, TrackProcessService},
        settings::LocationInfo,
    },
    shared::{io::domain::IoError, serializable_command::domain::SerializableCommand},
};

use super::TrackProcessParams;

/// A failed exit the user did not ask for. An unknown exit is not reported as a crash.
fn is_crash(status: Option<ExitStatus>, killed: bool) -> bool {
    !killed && status.is_some_and(|s| !s.success())
}

pub struct ManageProcessParams {
    pub process_id: Uuid,
    pub instance_id: InstanceId,
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

        let killed = self.process_storage.was_killed(process_id).await;
        self.process_storage.remove(process_id).await?;

        self.event_emitter
            .emit_safe(ProcessEvent {
                instance_id: instance_id.clone(),
                process_id,
                event: ProcessEventType::Finished,
                message: "Exited process".to_string(),
                exit_code: mc_exit_status.and_then(|s| s.code()),
                crashed: is_crash(mc_exit_status, killed),
            })
            .await;

        if mc_exit_status.is_some_and(|s| s.success()) && !post_exit_command.is_empty() {
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

    fn run_post_exit(&self, command: &str, instance_id: &InstanceId) -> Result<(), ProcessError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        if let Ok(cmd) = SerializableCommand::from_string(command, Some(&instance_dir)) {
            cmd.to_tokio_command()
                .spawn()
                .map_err(|e| IoError::with_path(e, instance_dir))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn status(code: i32) -> ExitStatus {
        #[cfg(windows)]
        {
            std::os::windows::process::ExitStatusExt::from_raw(code.cast_unsigned())
        }
        #[cfg(unix)]
        {
            std::os::unix::process::ExitStatusExt::from_raw(code << 8)
        }
    }

    #[rstest]
    #[case::nonzero_exit(Some(1), false, true)]
    #[case::clean_exit(Some(0), false, false)]
    #[case::killed_process(Some(1), true, false)]
    #[case::unknown_exit(None, false, false)]
    fn should_report_a_crash_only_for_an_unrequested_nonzero_exit(
        #[case] code: Option<i32>,
        #[case] killed: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(is_crash(code.map(status), killed), expected);
    }
}
