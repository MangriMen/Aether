use std::{process::Stdio, sync::Arc};

use async_trait::async_trait;
use process_wrap::tokio::{ChildWrapper, CommandWrap};
use tokio::process::Command;

use crate::features::{
    events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
    game_log::{CaptureGameOutputParams, GameOutputCaptureService},
    instance::InstanceId,
    process::{
        KillScope, ManageProcessService, MinecraftProcessMetadata, ProcessError,
        ProcessStartService, ProcessStorage,
    },
};

use super::ManageProcessParams;

pub struct StartProcessUseCase {
    event_emitter: SharedEventEmitter,
    process_storage: Arc<dyn ProcessStorage>,
    manage_process_service: Arc<dyn ManageProcessService>,
    game_output_capture: Arc<dyn GameOutputCaptureService>,
}

impl StartProcessUseCase {
    pub fn new(
        event_emitter: SharedEventEmitter,
        process_storage: Arc<dyn ProcessStorage>,
        manage_process_service: Arc<dyn ManageProcessService>,
        game_output_capture: Arc<dyn GameOutputCaptureService>,
    ) -> Self {
        Self {
            event_emitter,
            process_storage,
            manage_process_service,
            game_output_capture,
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
        let metadata = MinecraftProcessMetadata::new(instance_id.clone());

        let capture = self.game_output_capture.start(CaptureGameOutputParams {
            instance_id: instance_id.clone(),
            process_id: metadata.uuid(),
            stdout: process.stdout().take(),
            stderr: process.stderr().take(),
            secrets,
        });

        self.process_storage
            .insert(metadata.clone(), process)
            .await?;

        let manage_process_service = self.manage_process_service.clone();
        let instance_id_clone = instance_id.clone();
        let process_uuid_clone = metadata.uuid();

        tokio::spawn(async move {
            let _ = manage_process_service
                .execute(ManageProcessParams {
                    process_id: process_uuid_clone,
                    instance_id: instance_id_clone,
                    post_exit_command,
                    capture,
                })
                .await;
        });

        self.event_emitter
            .emit_safe(ProcessEvent {
                instance_id,
                process_id: metadata.uuid(),
                event: ProcessEventType::Launched,
                message: "Launched Minecraft".to_string(),
                exit_code: None,
                crashed: false,
                hook_error: None,
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

/// Start `command`, in a job object or process group of its own when a [`KillScope::Tree`] is
/// asked for. The game is never stopped on drop: it must outlive the launcher. Its end is still
/// the exit of the started process alone, so a descendant that outlives it is not waited for.
fn spawn(command: Command, kill_scope: KillScope) -> std::io::Result<Box<dyn ChildWrapper>> {
    let mut command = CommandWrap::from(command);
    if kill_scope == KillScope::Tree {
        // The job object replaces creation flags set on the command: any the game needs must be
        // given through `CreationFlags`.
        #[cfg(windows)]
        command.wrap(process_wrap::tokio::JobObject);
        #[cfg(unix)]
        command.wrap(process_wrap::tokio::ProcessGroup::leader());
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
}

#[cfg(test)]
mod spawn_error_tests {
    use crate::features::{
        events::{Event, EventEmitter, EventError},
        game_log::{CaptureGameOutputParams, CaptureHandle, GameOutputCaptureService},
        process::infra::InMemoryProcessStorage,
    };

    use super::*;

    struct NoEvents;

    #[async_trait]
    impl EventEmitter<Event> for NoEvents {
        async fn emit(&self, _event: Event) -> Result<(), EventError> {
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

    struct NotManaged;

    #[async_trait]
    impl ManageProcessService for NotManaged {
        async fn execute(&self, _params: ManageProcessParams) -> Result<(), ProcessError> {
            Ok(())
        }
    }

    struct NotCaptured;

    impl GameOutputCaptureService for NotCaptured {
        fn start(&self, _params: CaptureGameOutputParams) -> CaptureHandle {
            CaptureHandle::new(tokio::sync::oneshot::channel().1)
        }
    }

    #[tokio::test]
    async fn should_name_the_program_when_it_cannot_be_started() {
        let use_case = StartProcessUseCase::new(
            Arc::new(NoEvents),
            Arc::new(InMemoryProcessStorage::default()),
            Arc::new(NotManaged),
            Arc::new(NotCaptured),
        );
        let mut command = Command::new("aether-no-such-program-0f3a");
        command.arg("--accessToken").arg("secret");

        let result = use_case
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
}
