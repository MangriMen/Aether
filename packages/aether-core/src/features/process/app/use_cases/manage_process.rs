use std::{process::ExitStatus, sync::Arc, time::Duration};

use uuid::Uuid;

use crate::{
    features::{
        events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
        game_log::CaptureHandle,
        instance::InstanceId,
        process::{ManageProcessService, ProcessError, ProcessStorage, TrackProcessService},
        settings::LocationInfo,
    },
    shared::serializable_command::{HookKind, HookSpawnError, SerializableCommand},
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
    pub capture: CaptureHandle,
}

/// How long the last output may lag behind the exit before `Finished` is sent anyway.
const CAPTURE_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

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
            capture,
        } = params;

        let mc_exit_status = self
            .track_process_service
            .execute(TrackProcessParams {
                process_id,
                instance_id: instance_id.clone(),
            })
            .await;

        capture.wait(CAPTURE_DRAIN_TIMEOUT).await;

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

        if mc_exit_status.is_some_and(|s| s.success())
            && let Err(err) = self.run_post_exit(&post_exit_command, &instance_id)
        {
            tracing::warn!(instance_id = %instance_id, error = %err, "Post-exit hook failed");
            self.event_emitter
                .emit_safe(ProcessEvent {
                    instance_id,
                    process_id,
                    message: err.to_string(),
                    event: ProcessEventType::PostExitHookFailed(err),
                    exit_code: None,
                    crashed: false,
                })
                .await;
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

    /// Starts the post-exit hook `command` in the instance folder and leaves it running: the
    /// game is over, and nothing waits for the hook. An empty hook is no hook.
    fn run_post_exit(&self, command: &str, instance_id: &InstanceId) -> Result<(), HookSpawnError> {
        if command.trim().is_empty() {
            return Ok(());
        }
        let instance_dir = self.location_info.instance_dir(instance_id);
        let command =
            SerializableCommand::from_string(command, Some(&instance_dir)).map_err(|reason| {
                HookSpawnError::Invalid {
                    hook: HookKind::PostExit,
                    reason,
                }
            })?;
        command.to_tokio_command().spawn().map_err(|err| {
            HookSpawnError::spawn_failed(HookKind::PostExit, &command.program, err)
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use tokio::sync::{mpsc, oneshot};

    use super::*;
    use crate::features::{
        events::{Event, test_support::ChannelEmitter},
        process::infra::InMemoryProcessStorage,
    };
    use crate::shared::serializable_command::CommandParseError;

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

    struct ExitedWith(i32);

    #[async_trait::async_trait]
    impl TrackProcessService for ExitedWith {
        async fn execute(&self, _params: TrackProcessParams) -> Option<ExitStatus> {
            Some(status(self.0))
        }
    }

    /// Runs the end of a game that exited with `game_exit_code`, with the post-exit hook
    /// `post_exit`, and gives the process events it sent.
    async fn finish_game(game_exit_code: i32, post_exit: &str) -> Vec<ProcessEvent> {
        let dir = tempfile::tempdir().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            dir.path().join("settings"),
            dir.path().to_path_buf(),
        ));
        let instance_id = InstanceId::try_from("test-instance").unwrap();
        std::fs::create_dir_all(location_info.instance_dir(&instance_id)).unwrap();
        let (sender, mut events) = mpsc::unbounded_channel();
        let use_case = ManageProcessUseCase::new(
            Arc::new(ChannelEmitter(sender)),
            Arc::new(InMemoryProcessStorage::default()),
            Arc::new(ExitedWith(game_exit_code)),
            location_info,
        );
        // The output is drained already: the sender is gone.
        let (_, done) = oneshot::channel();

        use_case
            .execute(ManageProcessParams {
                process_id: Uuid::new_v4(),
                instance_id,
                post_exit_command: post_exit.to_owned(),
                capture: CaptureHandle::new(done),
            })
            .await
            .unwrap();

        std::iter::from_fn(|| events.try_recv().ok())
            .filter_map(|event| match event {
                Event::Process(event) => Some(event),
                _ => None,
            })
            .collect()
    }

    fn kinds(events: &[ProcessEvent]) -> Vec<&'static str> {
        events
            .iter()
            .map(|event| match event.event {
                ProcessEventType::Launched => "launched",
                ProcessEventType::Finished => "finished",
                ProcessEventType::PostExitHookFailed(_) => "post_exit_hook_failed",
            })
            .collect()
    }

    #[tokio::test]
    async fn should_report_the_post_exit_hook_after_finished_when_it_cannot_be_started() {
        let events = finish_game(0, "aether-no-such-program-0f3a --token secret").await;

        assert_eq!(kinds(&events), ["finished", "post_exit_hook_failed"]);
        let failed = &events[1];
        assert!(
            matches!(
                &failed.event,
                ProcessEventType::PostExitHookFailed(HookSpawnError::SpawnFailed {
                    hook: HookKind::PostExit,
                    program,
                    ..
                }) if program == "aether-no-such-program-0f3a"
            ),
            "{failed:?}"
        );
        assert!(failed.message.contains("aether-no-such-program-0f3a"));
        assert!(!failed.message.contains("secret"), "{}", failed.message);
    }

    #[tokio::test]
    async fn should_report_the_post_exit_hook_when_a_quote_of_it_is_not_closed() {
        let events = finish_game(0, "echo \"done").await;

        assert_eq!(kinds(&events), ["finished", "post_exit_hook_failed"]);
        assert!(matches!(
            events[1].event,
            ProcessEventType::PostExitHookFailed(HookSpawnError::Invalid {
                hook: HookKind::PostExit,
                reason: CommandParseError::UnclosedQuote,
            })
        ));
    }

    #[tokio::test]
    async fn should_not_report_a_post_exit_hook_that_started_and_failed() {
        #[cfg(windows)]
        let hook = "cmd /c exit 3";
        #[cfg(unix)]
        let hook = "sh -c \"exit 3\"";

        let events = finish_game(0, hook).await;

        assert_eq!(kinds(&events), ["finished"]);
    }

    #[tokio::test]
    async fn should_not_run_the_post_exit_hook_when_the_game_failed() {
        let events = finish_game(1, "aether-no-such-program-0f3a").await;

        assert_eq!(kinds(&events), ["finished"]);
    }
}
