use std::{process::Stdio, sync::Arc};

use async_trait::async_trait;
use process_wrap::tokio::{ChildWrapper, CommandWrap};
use tokio::process::Command;

use crate::{
    features::{
        events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
        game_log::{CaptureGameOutputParams, GameOutputCaptureService},
        instance::InstanceId,
        process::{
            KillScope, ManageProcessService, MinecraftProcessMetadata, ProcessError,
            ProcessStartService, ProcessStorage,
        },
    },
    shared::io::domain::IoError,
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

        let mut process = spawn(command, kill_scope).map_err(IoError::from)?;
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
    use std::{
        future::Future,
        path::PathBuf,
        time::{Duration, Instant},
    };

    use tokio::io::{AsyncBufReadExt, BufReader};

    use super::*;

    /// Where [`finish_when_told`] looks for its `go` file and leaves its `done` file.
    const SIGNAL_DIR_VAR: &str = "AETHER_TEST_SIGNAL_DIR";

    /// Not a test: a grandchild the tree tests start, which never ends.
    #[test]
    #[ignore = "started by the process tree tests as a grandchild that hangs"]
    fn print_pid_and_hang() {
        use std::io::Write;

        println!("pid:{}", std::process::id());
        std::io::stdout().flush().unwrap();
        std::thread::sleep(Duration::from_secs(600));
    }

    /// Not a test: a grandchild the tree tests start, which ends once a `go` file appears and
    /// leaves a `done` file to show it was still running then.
    #[test]
    #[ignore = "started by the process tree tests as a grandchild that waits for a file"]
    fn finish_when_told() {
        let dir = PathBuf::from(std::env::var_os(SIGNAL_DIR_VAR).unwrap());
        let deadline = Instant::now() + Duration::from_secs(60);
        while !dir.join("go").exists() {
            assert!(Instant::now() < deadline, "no go file appeared");
            std::thread::sleep(Duration::from_millis(20));
        }
        std::fs::write(dir.join("done"), "").unwrap();
    }

    /// One of the helpers above, started by a shell, so that it is a grandchild of the process
    /// started.
    fn behind_a_shell(helper: &str) -> Command {
        let module = module_path!().trim_start_matches("aether_core::");
        let exe = std::env::current_exe().unwrap();
        let args = ["--ignored", "--exact", "--nocapture"];
        let test = format!("{module}::{helper}");
        #[cfg(windows)]
        let mut command = {
            let mut command = Command::new("cmd");
            command.arg("/c").arg(exe).args(args).arg(test);
            command
        };
        #[cfg(target_os = "linux")]
        let mut command = {
            let mut command = Command::new("sh");
            command
                .args(["-c", r#""$0" "$@" & wait"#])
                .arg(exe)
                .args(args)
                .arg(test);
            command
        };
        command.stdin(Stdio::null()).stdout(Stdio::piped());
        command
    }

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

    async fn wait_until<F: Future<Output = bool>>(what: &str, mut check: impl FnMut() -> F) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while !check().await {
            assert!(Instant::now() < deadline, "timed out waiting until {what}");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
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

    #[cfg(windows)]
    async fn is_running(pid: u32) -> bool {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output()
            .await
            .unwrap();
        String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\""))
    }

    #[tokio::test]
    async fn should_stop_the_grandchild_when_the_tree_is_killed() {
        let mut child = spawn(behind_a_shell("print_pid_and_hang"), KillScope::Tree).unwrap();
        let grandchild = printed_pid(child.as_mut()).await;

        child.start_kill().unwrap();

        wait_until("the grandchild is gone", || async move {
            !is_running(grandchild).await
        })
        .await;
    }

    #[tokio::test]
    async fn should_leave_the_tree_running_when_the_launcher_lets_go_of_it() {
        let dir = tempfile::tempdir().unwrap();
        let mut command = behind_a_shell("finish_when_told");
        command
            .env(SIGNAL_DIR_VAR, dir.path())
            .stdout(Stdio::null());
        let child = spawn(command, KillScope::Tree).unwrap();

        // What the launcher's exit does to the game: its hold on the tree goes away.
        drop(child);
        std::fs::write(dir.path().join("go"), "").unwrap();

        let done = dir.path().join("done");
        wait_until("the grandchild answers", || {
            std::future::ready(done.exists())
        })
        .await;
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn should_keep_the_process_in_the_launcher_group_when_only_it_is_to_be_killed() {
        let mut command = Command::new("sleep");
        command.arg("60").stdin(Stdio::null());
        let mut child = spawn(command, KillScope::Process).unwrap();
        let pid = child.id().unwrap();

        let group = proc_stat(&pid.to_string()).map(|(_, group)| group);
        child.start_kill().unwrap();

        assert_eq!(group, proc_stat("self").map(|(_, group)| group));
    }
}
