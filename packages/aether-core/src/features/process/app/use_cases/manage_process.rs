use std::{process::ExitStatus, sync::Arc, time::Duration};

use uuid::Uuid;

use crate::{
    features::{
        events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
        game_log::CaptureHandle,
        instance::InstanceId,
        process::{
            ManageProcessService, ProcessError, ProcessOrigin, ProcessStorage, TrackProcessService,
            app::ports::{SkippedHookNoticeService, TrackedEnd},
        },
        settings::LocationInfo,
    },
    shared::serializable_command::{HookKind, HookSpawnError, parse_hook_command},
};

use super::TrackProcessParams;

/// A failed exit the user did not ask for. An unknown exit is not reported as a crash.
fn is_crash(status: Option<ExitStatus>, killed: bool) -> bool {
    !killed && status.is_some_and(|s| !s.success())
}

/// What becomes of the post-exit hook once the game has ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PostExit {
    Run,
    Skip,
    /// The hook is not run, and the user is told: the game was taken over from an earlier
    /// launcher, and how it ended cannot be read.
    SkipAndTell,
}

/// The hook runs after a clean exit only, and never after a stop the user asked for. An exit
/// that cannot be read is not taken for a clean one, and a game the launcher lost sight of may
/// not have ended at all.
fn post_exit(end: TrackedEnd, killed: bool, origin: ProcessOrigin) -> PostExit {
    if killed {
        return PostExit::Skip;
    }
    match end {
        TrackedEnd::Exited(Some(status)) if status.success() => PostExit::Run,
        TrackedEnd::Exited(Some(_)) | TrackedEnd::Lost => PostExit::Skip,
        TrackedEnd::Exited(None) => match origin {
            ProcessOrigin::Adopted => PostExit::SkipAndTell,
            ProcessOrigin::Launched => PostExit::Skip,
        },
    }
}

pub struct ManageProcessParams {
    pub process_id: Uuid,
    pub instance_id: InstanceId,
    pub post_exit_command: String,
    /// `None` for a game taken over from an earlier launcher: its output went with that one.
    pub capture: Option<CaptureHandle>,
    pub origin: ProcessOrigin,
}

/// How long the last output may lag behind the exit before `Finished` is sent anyway.
const CAPTURE_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

pub struct ManageProcessUseCase {
    event_emitter: SharedEventEmitter,
    process_storage: Arc<dyn ProcessStorage>,
    track_process_service: Arc<dyn TrackProcessService>,
    location_info: Arc<LocationInfo>,
    notices: Arc<dyn SkippedHookNoticeService>,
}

#[async_trait::async_trait]
impl ManageProcessService for ManageProcessUseCase {
    async fn execute(&self, params: ManageProcessParams) -> Result<(), ProcessError> {
        let ManageProcessParams {
            process_id,
            instance_id,
            post_exit_command,
            capture,
            origin,
        } = params;

        let end = self
            .track_process_service
            .execute(TrackProcessParams {
                process_id,
                instance_id: instance_id.clone(),
            })
            .await;
        let mc_exit_status = match end {
            TrackedEnd::Exited(status) => status,
            TrackedEnd::Lost => {
                tracing::warn!(
                    %process_id,
                    %instance_id,
                    "Lost sight of the game: it is no longer followed, and its record stays"
                );
                None
            }
        };

        if let Some(capture) = capture {
            capture.wait(CAPTURE_DRAIN_TIMEOUT).await;
        }

        let killed = self.process_storage.was_killed(process_id).await;
        self.process_storage.remove(process_id).await?;

        let adopted = origin == ProcessOrigin::Adopted;
        self.event_emitter
            .emit_safe(ProcessEvent {
                instance_id: instance_id.clone(),
                process_id,
                event: ProcessEventType::Finished,
                message: "Exited process".to_string(),
                exit_code: mc_exit_status.and_then(|s| s.code()),
                crashed: is_crash(mc_exit_status, killed),
                adopted,
            })
            .await;

        match post_exit(end, killed, origin) {
            PostExit::Run => {
                if let Err(err) = self.run_post_exit(&post_exit_command, &instance_id) {
                    tracing::warn!(instance_id = %instance_id, error = %err, "Post-exit hook failed");
                    self.event_emitter
                        .emit_safe(ProcessEvent {
                            instance_id,
                            process_id,
                            message: err.to_string(),
                            event: ProcessEventType::PostExitHookFailed(err),
                            exit_code: None,
                            crashed: false,
                            adopted,
                        })
                        .await;
                }
            }
            PostExit::SkipAndTell if !post_exit_command.trim().is_empty() => {
                tracing::info!(
                    %process_id,
                    %instance_id,
                    "The post-exit hook was not run: how the taken over game ended is unknown"
                );
                self.notices.notify(&instance_id).await;
            }
            PostExit::SkipAndTell | PostExit::Skip => {}
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
        notices: Arc<dyn SkippedHookNoticeService>,
    ) -> Self {
        Self {
            event_emitter,
            process_storage,
            track_process_service,
            location_info,
            notices,
        }
    }

    /// Starts the post-exit hook `command` in the instance folder and leaves it running: the
    /// game is over, and nothing waits for the hook. An empty hook is no hook.
    fn run_post_exit(&self, command: &str, instance_id: &InstanceId) -> Result<(), HookSpawnError> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        let Some(command) = parse_hook_command(HookKind::PostExit, command, &instance_dir)? else {
            return Ok(());
        };
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
        events::{Event, WarningEvent, test_support::ChannelEmitter},
        instance::{Instance, InstanceChange, InstanceError, InstanceStorage},
        process::{
            AdoptedProcess, MinecraftProcessMetadata, ProcessHandle, ProcessState,
            TakeSkippedHookNoticesUseCase, app::ports::SkippedHookNoticeService,
            infra::InMemoryProcessStorage,
        },
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

    #[rstest]
    #[case::launched_clean_exit(exited(Some(0)), false, ProcessOrigin::Launched, PostExit::Run)]
    #[case::adopted_clean_exit_with_a_code(
        exited(Some(0)),
        false,
        ProcessOrigin::Adopted,
        PostExit::Run
    )]
    #[case::adopted_failed_exit(exited(Some(1)), false, ProcessOrigin::Adopted, PostExit::Skip)]
    #[case::launched_unknown_exit(exited(None), false, ProcessOrigin::Launched, PostExit::Skip)]
    #[case::adopted_unknown_exit(
        exited(None),
        false,
        ProcessOrigin::Adopted,
        PostExit::SkipAndTell
    )]
    #[case::killed_clean_exit(exited(Some(0)), true, ProcessOrigin::Launched, PostExit::Skip)]
    #[case::killed_adopted_clean_exit(
        exited(Some(0)),
        true,
        ProcessOrigin::Adopted,
        PostExit::Skip
    )]
    #[case::killed_adopted_unknown_exit(exited(None), true, ProcessOrigin::Adopted, PostExit::Skip)]
    #[case::lost_adopted_game(TrackedEnd::Lost, false, ProcessOrigin::Adopted, PostExit::Skip)]
    fn should_run_the_post_exit_hook_only_after_a_clean_exit_that_was_not_a_stop(
        #[case] end: TrackedEnd,
        #[case] killed: bool,
        #[case] origin: ProcessOrigin,
        #[case] expected: PostExit,
    ) {
        assert_eq!(post_exit(end, killed, origin), expected);
    }

    fn exited(code: Option<i32>) -> TrackedEnd {
        TrackedEnd::Exited(code.map(status))
    }

    struct EndedAs(TrackedEnd);

    #[async_trait::async_trait]
    impl TrackProcessService for EndedAs {
        async fn execute(&self, _params: TrackProcessParams) -> TrackedEnd {
            self.0
        }
    }

    /// Knows no instance: a notice names the instance by its id.
    struct NoInstances;

    #[async_trait::async_trait]
    impl InstanceStorage for NoInstances {
        async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
            unimplemented!()
        }

        async fn get(&self, _id: &InstanceId) -> Result<Instance, InstanceError> {
            Err(InstanceError::NotFound {
                instance_id: "test-instance".to_owned(),
            })
        }

        async fn upsert(&self, _instance: &Instance) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn remove(&self, _id: &InstanceId) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn update(
            &self,
            _id: &InstanceId,
            _change: InstanceChange<'_>,
        ) -> Result<Instance, InstanceError> {
            unimplemented!()
        }
    }

    /// A taken over game that takes the stop signal.
    #[derive(Debug)]
    struct Stoppable;

    impl AdoptedProcess for Stoppable {
        fn try_wait(&mut self) -> std::io::Result<ProcessState> {
            Ok(ProcessState::Running)
        }

        fn start_kill(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    struct Ending {
        end: TrackedEnd,
        origin: ProcessOrigin,
        post_exit: &'static str,
        killed: bool,
    }

    impl Ending {
        fn launched(exit_code: i32, post_exit: &'static str) -> Self {
            Self {
                end: exited(Some(exit_code)),
                origin: ProcessOrigin::Launched,
                post_exit,
                killed: false,
            }
        }
    }

    struct Finished {
        events: Vec<ProcessEvent>,
        notices: Vec<WarningEvent>,
    }

    /// Runs the end of a game as `ending` tells, and gives the process events it sent and the
    /// notices it left.
    async fn finish(ending: Ending) -> Finished {
        let dir = tempfile::tempdir().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            dir.path().join("settings"),
            dir.path().to_path_buf(),
        ));
        let instance_id = InstanceId::try_from("test-instance").unwrap();
        std::fs::create_dir_all(location_info.instance_dir(&instance_id)).unwrap();
        let (sender, mut events) = mpsc::unbounded_channel();
        let emitter: SharedEventEmitter = Arc::new(ChannelEmitter(sender));
        let storage = Arc::new(InMemoryProcessStorage::default());
        let metadata = MinecraftProcessMetadata::new(instance_id.clone());
        let process_id = metadata.uuid();
        if ending.killed {
            storage
                .insert(metadata, ProcessHandle::Adopted(Box::new(Stoppable)))
                .await
                .unwrap();
            storage.kill(process_id).await.unwrap();
        }
        let notices = Arc::new(TakeSkippedHookNoticesUseCase::new(
            emitter.clone(),
            Arc::new(NoInstances),
        ));
        let use_case = ManageProcessUseCase::new(
            emitter,
            storage,
            Arc::new(EndedAs(ending.end)),
            location_info,
            notices.clone(),
        );
        let capture = match ending.origin {
            // The output is drained already: the sender is gone.
            ProcessOrigin::Launched => Some(CaptureHandle::new(oneshot::channel().1)),
            ProcessOrigin::Adopted => None,
        };

        use_case
            .execute(ManageProcessParams {
                process_id,
                instance_id,
                post_exit_command: ending.post_exit.to_owned(),
                capture,
                origin: ending.origin,
            })
            .await
            .unwrap();

        let events = std::iter::from_fn(|| events.try_recv().ok())
            .filter_map(|event| match event {
                Event::Process(event) => Some(event),
                _ => None,
            })
            .collect();
        Finished {
            events,
            notices: notices.execute(),
        }
    }

    async fn finish_game(game_exit_code: i32, post_exit: &'static str) -> Vec<ProcessEvent> {
        finish(Ending::launched(game_exit_code, post_exit))
            .await
            .events
    }

    fn kinds(events: &[ProcessEvent]) -> Vec<&'static str> {
        events
            .iter()
            .map(|event| match event.event {
                ProcessEventType::Launched => "launched",
                ProcessEventType::Adopted => "adopted",
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

    #[tokio::test]
    async fn should_run_the_post_exit_hook_when_a_taken_over_game_exits_cleanly_with_a_code() {
        let finished = finish(Ending {
            origin: ProcessOrigin::Adopted,
            ..Ending::launched(0, "aether-no-such-program-0f3a")
        })
        .await;

        assert_eq!(
            kinds(&finished.events),
            ["finished", "post_exit_hook_failed"]
        );
        assert!(finished.events.iter().all(|event| event.adopted));
        assert!(finished.notices.is_empty());
    }

    #[tokio::test]
    async fn should_tell_of_the_skipped_hook_when_a_taken_over_game_ends_in_an_unknown_way() {
        let finished = finish(Ending {
            end: exited(None),
            origin: ProcessOrigin::Adopted,
            ..Ending::launched(0, "aether-no-such-program-0f3a")
        })
        .await;

        assert_eq!(kinds(&finished.events), ["finished"]);
        assert!(!finished.events[0].crashed);
        assert_eq!(finished.events[0].exit_code, None);
        assert!(
            matches!(
                finished.notices.as_slice(),
                [WarningEvent::PostExitHookSkipped { instance_id, instance_name }]
                    if instance_id.as_str() == "test-instance" && instance_name == "test-instance"
            ),
            "{:?}",
            finished.notices
        );
    }

    #[rstest]
    #[case::stopped_by_the_user("aether-no-such-program-0f3a", true)]
    #[case::no_hook("  ", false)]
    #[tokio::test]
    async fn should_tell_nothing_when_a_taken_over_game_ends_in_an_unknown_way_and_no_hook_was_due(
        #[case] post_exit: &'static str,
        #[case] killed: bool,
    ) {
        let finished = finish(Ending {
            end: exited(None),
            origin: ProcessOrigin::Adopted,
            post_exit,
            killed,
        })
        .await;

        assert_eq!(kinds(&finished.events), ["finished"]);
        assert!(finished.notices.is_empty(), "{:?}", finished.notices);
    }

    #[tokio::test]
    async fn should_tell_nothing_when_the_launcher_loses_sight_of_a_taken_over_game() {
        let finished = finish(Ending {
            end: TrackedEnd::Lost,
            origin: ProcessOrigin::Adopted,
            ..Ending::launched(0, "aether-no-such-program-0f3a")
        })
        .await;

        assert_eq!(kinds(&finished.events), ["finished"]);
        assert!(!finished.events[0].crashed);
        assert!(finished.notices.is_empty(), "{:?}", finished.notices);
    }

    #[tokio::test]
    async fn should_hold_the_notices_only_until_they_are_first_taken() {
        let (sender, mut events) = mpsc::unbounded_channel();
        let notices = TakeSkippedHookNoticesUseCase::new(
            Arc::new(ChannelEmitter(sender)),
            Arc::new(NoInstances),
        );
        let instance_id = InstanceId::try_from("test-instance").unwrap();

        notices.notify(&instance_id).await;
        let held = notices.execute();
        notices.notify(&instance_id).await;
        let held_later = notices.execute();

        assert_eq!(held.len(), 1);
        assert!(held_later.is_empty());
        let sent: Vec<_> = std::iter::from_fn(|| events.try_recv().ok()).collect();
        assert!(
            matches!(
                sent.as_slice(),
                [Event::Warning(WarningEvent::PostExitHookSkipped { .. })]
            ),
            "{sent:?}"
        );
    }
}
