use std::{process::Stdio, sync::Arc};

use async_trait::async_trait;
use process_wrap::tokio::{ChildWrapper, CommandWrap};
use tokio::process::Command;

use crate::core::domain::error_chain;
use crate::features::{
    events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
    game_log::{CaptureGameOutputParams, GameOutputCaptureService},
    instance::InstanceId,
    process::{
        KillScope, ManageProcessService, MinecraftProcessMetadata, ProcessError, ProcessHandle,
        ProcessOrigin, ProcessProbe, ProcessRecord, ProcessRecordStorage, ProcessStartService,
        ProcessStorage,
    },
};

use crate::shared::process_tree::wrap_tree;

use super::ManageProcessParams;

pub struct StartProcessUseCase {
    event_emitter: SharedEventEmitter,
    process_storage: Arc<dyn ProcessStorage>,
    manage_process_service: Arc<dyn ManageProcessService>,
    game_output_capture: Arc<dyn GameOutputCaptureService>,
    records: Arc<dyn ProcessRecordStorage>,
    process_probe: Arc<dyn ProcessProbe>,
}

impl StartProcessUseCase {
    pub fn new(
        event_emitter: SharedEventEmitter,
        process_storage: Arc<dyn ProcessStorage>,
        manage_process_service: Arc<dyn ManageProcessService>,
        game_output_capture: Arc<dyn GameOutputCaptureService>,
        records: Arc<dyn ProcessRecordStorage>,
        process_probe: Arc<dyn ProcessProbe>,
    ) -> Self {
        Self {
            event_emitter,
            process_storage,
            manage_process_service,
            game_output_capture,
            records,
            process_probe,
        }
    }

    pub async fn execute(
        &self,
        instance_id: InstanceId,
        mut command: Command,
        kill_scope: KillScope,
        post_exit_command: String,
        secrets: Vec<String>,
    ) -> Result<MinecraftProcessMetadata, ProcessError> {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let program = command
            .as_std()
            .get_program()
            .to_string_lossy()
            .into_owned();
        let mut process = spawn(command, kill_scope).map_err(|err| ProcessError::SpawnFailed {
            program,
            source: Arc::new(err),
        })?;
        let pid = process.id();
        let metadata = MinecraftProcessMetadata::new(instance_id.clone());

        let capture = self.game_output_capture.start(CaptureGameOutputParams {
            instance_id: instance_id.clone(),
            process_id: metadata.uuid(),
            stdout: process.stdout().take(),
            stderr: process.stderr().take(),
            secrets,
        });

        self.process_storage
            .insert(metadata.clone(), ProcessHandle::Owned(process))
            .await?;
        let manage_process_service = self.manage_process_service.clone();
        let records = self.records.clone();
        let process_probe = self.process_probe.clone();
        let started = metadata.clone();
        let instance_id_clone = instance_id.clone();
        let process_uuid_clone = metadata.uuid();

        tokio::spawn(async move {
            // Off the launch, which holds the instance lock, but before the game is followed:
            // its end removes the record, and a record saved after that would outlive the game.
            save_record(
                records.as_ref(),
                process_probe.as_ref(),
                &started,
                pid,
                kill_scope,
                &post_exit_command,
            )
            .await;
            if let Err(err) = manage_process_service
                .execute(ManageProcessParams {
                    process_id: process_uuid_clone,
                    instance_id: instance_id_clone,
                    post_exit_command,
                    capture: Some(capture),
                    log_tail: None,
                    origin: ProcessOrigin::Launched,
                    crash_watch: None,
                })
                .await
            {
                tracing::error!(process_id = %process_uuid_clone, "Could not follow the game process: {err}");
            }
        });

        self.event_emitter
            .emit_safe(ProcessEvent {
                instance_id,
                process_id: metadata.uuid(),
                event: ProcessEventType::Launched,
                message: "Launched Minecraft".to_string(),
                exit_code: None,
                crashed: false,
                adopted: false,
            })
            .await;

        Ok(metadata)
    }
}

#[async_trait]
impl ProcessStartService for StartProcessUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
        command: Command,
        kill_scope: KillScope,
        post_exit_command: String,
        secrets: Vec<String>,
    ) -> Result<MinecraftProcessMetadata, ProcessError> {
        self.execute(instance_id, command, kill_scope, post_exit_command, secrets)
            .await
    }
}

/// Keeps the started game so that a later launcher finds it. A game that cannot be kept still
/// runs and is followed as before: it is only lost to a restart of the launcher.
async fn save_record(
    records: &dyn ProcessRecordStorage,
    probe: &dyn ProcessProbe,
    metadata: &MinecraftProcessMetadata,
    pid: Option<u32>,
    kill_scope: KillScope,
    post_exit_command: &str,
) {
    let process_id = metadata.uuid();
    let instance_id = metadata.instance_id();
    // The probe has logged why it does not know the launcher.
    let Some(owner) = probe.own_identity() else {
        return;
    };
    let Some(pid) = pid else {
        tracing::warn!(
            %process_id,
            %instance_id,
            "Could not save the started game: it has no pid"
        );
        return;
    };
    let process = match probe.identity_of(pid).await {
        Ok(process) => process,
        Err(err) => {
            tracing::warn!(
                %process_id,
                %instance_id,
                pid,
                error = %err,
                "Could not save the started game: the OS did not report its start"
            );
            return;
        }
    };
    let record = ProcessRecord {
        id: process_id,
        instance_id: instance_id.clone(),
        process,
        started_at: *metadata.start_time(),
        kill_scope,
        post_exit_command: post_exit_command.to_owned(),
        playtime_credited_until: *metadata.start_time(),
        owner: owner.clone(),
    };
    if let Err(err) = records.insert(&record).await {
        tracing::warn!(
            %process_id,
            %instance_id,
            error = %error_chain(&err),
            "Could not save the started game"
        );
    }
}

/// Start `command`, in a job object or process group of its own when a [`KillScope::Tree`] is
/// asked for. The game is never stopped on drop: it must outlive the launcher. Its end is still
/// the exit of the started process alone, so a descendant that outlives it is not waited for.
fn spawn(command: Command, kill_scope: KillScope) -> std::io::Result<Box<dyn ChildWrapper>> {
    let mut command = CommandWrap::from(command);
    if kill_scope == KillScope::Tree {
        wrap_tree(&mut command);
    }
    command.spawn()
}

// Process liveness is read from /proc on Linux; there is no such check for other Unixes.
#[cfg(all(test, any(target_os = "linux", windows)))]
mod tests {
    use std::time::{Duration, Instant};

    use tokio::io::{AsyncBufReadExt, BufReader};

    use super::*;

    /// Read the pid that the started process prints on a line `pid:<pid>`.
    async fn printed_pid(child: &mut dyn ChildWrapper) -> u32 {
        let stdout = child.stdout().take().expect("stdout is piped");
        let mut lines = BufReader::new(stdout).lines();
        let read = async {
            while let Some(line) = lines.next_line().await.unwrap() {
                if let Some(pid) = line.trim().strip_prefix("pid:") {
                    return pid.parse().unwrap();
                }
            }
            panic!("the process ended without printing a pid");
        };
        tokio::time::timeout(Duration::from_secs(30), read)
            .await
            .expect("no pid was printed")
    }

    async fn wait_until_gone(pid: u32) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while is_running(pid).await {
            assert!(Instant::now() < deadline, "process {pid} is still running");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    fn piped(mut command: Command) -> Command {
        command.stdin(Stdio::null()).stdout(Stdio::piped());
        command
    }

    /// The `(state, process group)` of a process; a reaped process has none.
    #[cfg(target_os = "linux")]
    fn proc_stat(pid: &str) -> Option<(char, i32)> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        // The name in parentheses may hold spaces; the fields after it do not.
        let mut fields = stat[stat.rfind(')')? + 1..].split_whitespace();
        let state = fields.next()?.chars().next()?;
        let group = fields.nth(1)?.parse().ok()?;
        Some((state, group))
    }

    /// A zombie is gone too: its parent may not have reaped it yet.
    #[cfg(target_os = "linux")]
    async fn is_running(pid: u32) -> bool {
        proc_stat(&pid.to_string()).is_some_and(|(state, _)| state != 'Z')
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn should_stop_the_grandchild_when_the_tree_is_killed() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 60 & echo pid:$!; wait"]);
        let mut child = spawn(piped(command), KillScope::Tree).unwrap();
        let grandchild = printed_pid(child.as_mut()).await;

        child.start_kill().unwrap();

        wait_until_gone(grandchild).await;
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn should_keep_the_process_in_the_launcher_group_when_only_it_is_to_be_killed() {
        let mut command = Command::new("sleep");
        command.arg("60");
        let mut child = spawn(piped(command), KillScope::Process).unwrap();
        let pid = child.id().unwrap();

        let group = proc_stat(&pid.to_string()).map(|(_, group)| group);
        child.start_kill().unwrap();

        assert_eq!(group, proc_stat("self").map(|(_, group)| group));
    }

    /// Not a test: the grandchild the Windows tree test starts, which never ends.
    #[cfg(windows)]
    #[test]
    #[ignore = "started by the process tree test as a grandchild that hangs"]
    fn print_pid_and_hang() {
        use std::io::Write;

        println!("pid:{}", std::process::id());
        std::io::stdout().flush().unwrap();
        std::thread::sleep(Duration::from_secs(600));
    }

    #[cfg(windows)]
    async fn is_running(pid: u32) -> bool {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output()
            .await
            .unwrap();
        String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\""))
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn should_stop_the_grandchild_when_the_tree_is_killed() {
        let module = module_path!().trim_start_matches("aether_core::");
        let mut command = Command::new("cmd");
        command
            .arg("/c")
            .arg(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "--nocapture"])
            .arg(format!("{module}::print_pid_and_hang"));
        let mut child = spawn(piped(command), KillScope::Tree).unwrap();
        let grandchild = printed_pid(child.as_mut()).await;

        child.start_kill().unwrap();

        wait_until_gone(grandchild).await;
    }

    /// A long-running program that has nothing to do with the test binary.
    fn long_running() -> Command {
        #[cfg(windows)]
        let mut command = Command::new("ping");
        #[cfg(windows)]
        command.args(["-n", "120", "127.0.0.1"]);
        #[cfg(not(windows))]
        let mut command = Command::new("sleep");
        #[cfg(not(windows))]
        command.arg("120");
        command
    }

    async fn force_kill(pid: u32) {
        #[cfg(windows)]
        let mut kill = Command::new("taskkill");
        #[cfg(windows)]
        kill.args(["/F", "/PID", &pid.to_string()]);
        #[cfg(not(windows))]
        let mut kill = Command::new("kill");
        #[cfg(not(windows))]
        kill.args(["-9", &pid.to_string()]);
        let _ = kill.output().await;
    }

    /// Dropping the handle is what the end of the launcher does to it: the job object or the
    /// process handle is closed. The game must stay running, a later launcher adopts it.
    async fn assert_outlives_dropped_handle(kill_scope: KillScope) {
        let mut command = long_running();
        command.stdin(Stdio::null()).stdout(Stdio::null());
        let child = spawn(command, kill_scope).unwrap();
        let pid = child.id().unwrap();

        drop(child);
        tokio::time::sleep(Duration::from_millis(500)).await;
        let survived = is_running(pid).await;
        force_kill(pid).await;

        assert!(survived, "the game ended with its handle ({kill_scope:?})");
    }

    #[tokio::test]
    async fn should_keep_the_game_running_when_its_handle_is_dropped() {
        assert_outlives_dropped_handle(KillScope::Process).await;
    }

    #[tokio::test]
    async fn should_keep_the_game_tree_running_when_its_handle_is_dropped() {
        assert_outlives_dropped_handle(KillScope::Tree).await;
    }
}

#[cfg(test)]
mod start_tests {
    use std::sync::Mutex;

    use chrono::{DateTime, Utc};
    use tokio::sync::oneshot;
    use uuid::Uuid;

    use crate::features::{
        events::test_support::NoOpEmitter,
        game_log::{CaptureGameOutputParams, CaptureHandle, GameOutputCaptureService},
        process::{AdoptedProcess, OsProcessIdentity, ProcessState},
    };
    use crate::shared::error::DatabaseError;

    use super::*;

    /// Keeps the metadata of the started games; the processes are left to end by themselves.
    #[derive(Default)]
    struct Started(Mutex<Vec<MinecraftProcessMetadata>>);

    #[async_trait]
    impl ProcessStorage for Started {
        async fn insert(
            &self,
            process: MinecraftProcessMetadata,
            _handle: ProcessHandle,
        ) -> Result<(), ProcessError> {
            self.0.lock().unwrap().push(process);
            Ok(())
        }

        async fn remove(&self, _id: Uuid) -> Result<(), ProcessError> {
            unimplemented!()
        }

        async fn replace_pending(
            &self,
            _id: Uuid,
            _handle: ProcessHandle,
        ) -> Result<bool, ProcessError> {
            unimplemented!()
        }

        async fn list_metadata(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError> {
            unimplemented!()
        }

        async fn get_metadata(
            &self,
            _id: Uuid,
        ) -> Result<Option<MinecraftProcessMetadata>, ProcessError> {
            unimplemented!()
        }

        async fn try_wait(&self, _id: Uuid) -> Result<Option<ProcessState>, ProcessError> {
            unimplemented!()
        }

        async fn kill(&self, _id: Uuid) -> Result<(), ProcessError> {
            unimplemented!()
        }

        async fn was_killed(&self, _id: Uuid) -> bool {
            unimplemented!()
        }
    }

    /// Tells, once it is asked to follow the game, whether the record of the game was there.
    struct Followed {
        records: Arc<Records>,
        record_was_saved: Mutex<Option<oneshot::Sender<bool>>>,
    }

    #[async_trait]
    impl ManageProcessService for Followed {
        async fn execute(&self, params: ManageProcessParams) -> Result<(), ProcessError> {
            let saved = self
                .records
                .saved
                .lock()
                .unwrap()
                .iter()
                .any(|record| record.id == params.process_id);
            if let Some(sender) = self.record_was_saved.lock().unwrap().take() {
                let _ = sender.send(saved);
            }
            Ok(())
        }
    }

    struct NotCaptured;

    impl GameOutputCaptureService for NotCaptured {
        fn start(&self, _params: CaptureGameOutputParams) -> CaptureHandle {
            CaptureHandle::new(oneshot::channel().1)
        }
    }

    /// Keeps the inserted records, or fails every write when `failing`.
    #[derive(Default)]
    struct Records {
        failing: bool,
        saved: Mutex<Vec<ProcessRecord>>,
    }

    #[async_trait]
    impl ProcessRecordStorage for Records {
        async fn insert(&self, record: &ProcessRecord) -> Result<(), DatabaseError> {
            if self.failing {
                return Err(DatabaseError::new("database is locked"));
            }
            self.saved.lock().unwrap().push(record.clone());
            Ok(())
        }

        async fn remove(&self, _id: Uuid) -> Result<bool, DatabaseError> {
            unimplemented!()
        }

        async fn list(&self) -> Result<Vec<ProcessRecord>, DatabaseError> {
            unimplemented!()
        }

        async fn list_for_instance(
            &self,
            _instance_id: &InstanceId,
        ) -> Result<Vec<ProcessRecord>, DatabaseError> {
            unimplemented!()
        }

        async fn touch_playtime(
            &self,
            _id: Uuid,
            _until: DateTime<Utc>,
        ) -> Result<(), DatabaseError> {
            unimplemented!()
        }

        async fn take_over(
            &self,
            _id: Uuid,
            _from: &OsProcessIdentity,
            _to: &OsProcessIdentity,
        ) -> Result<bool, DatabaseError> {
            unimplemented!()
        }
    }

    fn launcher() -> OsProcessIdentity {
        OsProcessIdentity::new(1, 5, "boot".to_owned())
    }

    /// Reports every process as started at stamp 7 of the boot of the launcher, and the
    /// launcher itself as `own`.
    struct Probe {
        own: Option<OsProcessIdentity>,
    }

    #[async_trait]
    impl ProcessProbe for Probe {
        async fn identity_of(&self, pid: u32) -> std::io::Result<OsProcessIdentity> {
            Ok(OsProcessIdentity::new(pid, 7, "boot".to_owned()))
        }

        fn own_identity(&self) -> Option<&OsProcessIdentity> {
            self.own.as_ref()
        }

        fn tells_exit_of_adopted(&self) -> bool {
            true
        }

        async fn adopt(
            &self,
            _record: &ProcessRecord,
        ) -> std::io::Result<Option<Box<dyn AdoptedProcess>>> {
            unimplemented!()
        }

        async fn is_alive(&self, _process: &OsProcessIdentity) -> std::io::Result<bool> {
            unimplemented!()
        }
    }

    struct Fixture {
        use_case: StartProcessUseCase,
        started: Arc<Started>,
        records: Arc<Records>,
        record_was_saved: oneshot::Receiver<bool>,
    }

    fn fixture(records: Records, own: Option<OsProcessIdentity>) -> Fixture {
        let started = Arc::new(Started::default());
        let records = Arc::new(records);
        let (sender, record_was_saved) = oneshot::channel();
        let use_case = StartProcessUseCase::new(
            Arc::new(NoOpEmitter),
            started.clone(),
            Arc::new(Followed {
                records: records.clone(),
                record_was_saved: Mutex::new(Some(sender)),
            }),
            Arc::new(NotCaptured),
            records.clone(),
            Arc::new(Probe { own }),
        );
        Fixture {
            use_case,
            started,
            records,
            record_was_saved,
        }
    }

    /// A program that is there on every platform and ends at once.
    fn quick_program() -> Command {
        if cfg!(windows) {
            let mut command = Command::new("cmd");
            command.args(["/c", "exit 0"]);
            command
        } else {
            Command::new("true")
        }
    }

    async fn start(
        use_case: &StartProcessUseCase,
    ) -> Result<MinecraftProcessMetadata, ProcessError> {
        use_case
            .execute(
                InstanceId::try_from("test-instance").unwrap(),
                quick_program(),
                KillScope::Process,
                "backup.sh".to_owned(),
                Vec::new(),
            )
            .await
    }

    #[tokio::test]
    async fn should_name_the_program_when_it_cannot_be_started() {
        let fixture = fixture(Records::default(), Some(launcher()));
        let mut command = Command::new("aether-no-such-program-0f3a");
        command.arg("--accessToken").arg("secret");

        let result = fixture
            .use_case
            .execute(
                InstanceId::try_from("test-instance").unwrap(),
                command,
                KillScope::Tree,
                String::new(),
                Vec::new(),
            )
            .await;

        let Err(err) = result else {
            panic!("the program was started");
        };
        assert!(
            matches!(
                &err,
                ProcessError::SpawnFailed { program, .. } if program == "aether-no-such-program-0f3a"
            ),
            "{err:?}"
        );
        assert!(!err.to_string().contains("secret"), "{err}");
    }

    #[tokio::test]
    async fn should_save_the_game_with_the_launcher_as_its_owner_before_it_is_followed() {
        let fixture = fixture(Records::default(), Some(launcher()));

        let metadata = start(&fixture.use_case).await.unwrap();

        assert!(
            fixture.record_was_saved.await.unwrap(),
            "the game was followed before its record was saved"
        );
        let saved = fixture.records.saved.lock().unwrap().clone();
        assert_eq!(saved.len(), 1);
        let record = &saved[0];
        assert_eq!(record.id, metadata.uuid());
        assert_eq!(&record.instance_id, metadata.instance_id());
        assert_eq!(record.process.start_stamp(), 7);
        assert!(record.owner.matches(&launcher()));
        assert_eq!(record.kill_scope, KillScope::Process);
        assert_eq!(record.post_exit_command, "backup.sh");
        assert_eq!(&record.started_at, metadata.start_time());
        assert_eq!(&record.playtime_credited_until, metadata.start_time());
    }

    #[tokio::test]
    async fn should_start_and_follow_the_game_when_its_record_cannot_be_saved() {
        let fixture = fixture(
            Records {
                failing: true,
                ..Records::default()
            },
            Some(launcher()),
        );

        let metadata = start(&fixture.use_case).await.unwrap();

        assert!(!fixture.record_was_saved.await.unwrap());
        let started = fixture.started.0.lock().unwrap();
        assert!(
            started
                .iter()
                .any(|process| process.uuid() == metadata.uuid())
        );
    }

    #[tokio::test]
    async fn should_save_no_record_when_the_launcher_does_not_know_its_own_process() {
        let fixture = fixture(Records::default(), None);

        start(&fixture.use_case).await.unwrap();

        assert!(!fixture.record_was_saved.await.unwrap());
        assert!(fixture.records.saved.lock().unwrap().is_empty());
    }
}
