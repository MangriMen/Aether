//! `log`, `get_id` and `run_command` — the core host functions exposed to plugins.
//!
//! `run_command` is the one place where a plugin can start a process on the host, so it is
//! fenced in on four sides:
//!
//! * only a program next to a Java runtime the core knows about, by canonical path
//!   ([`plugin_utils::plugin_command_to_host`]);
//! * the working directory is mandatory and must sit inside a directory mounted for the plugin;
//! * the child gets an explicit minimal environment rather than the launcher's own;
//! * the run is capped in both wall-clock time and captured output, and what it started is
//!   stopped as a whole process tree when it runs out of time or its call is cancelled.

use std::{process::Stdio, time::Duration};

use aether_core_plugin_api::v0::{CommandDto, OutputDto};
use extism::host_fn;
use extism_convert::Msgpack;
use process_wrap::tokio::{ChildWrapper, CommandWrap};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    core::app::AetherContainer,
    features::{
        instance::PluginId,
        java::{Java, JavaFeature},
    },
    shared::{limits::COMMAND_TIMEOUT, serializable_command::domain::SerializableCommand},
};

use super::super::{
    super::{
        super::plugin_utils,
        mappers::{OutputDtoExt, to_extism_res},
    },
    PluginContext, block_on,
};

// ── Testable business logic ──

fn log_level_from_u32(level: u32) -> log::Level {
    match level {
        1 => log::Level::Error,
        2 => log::Level::Warn,
        3 => log::Level::Info,
        4 => log::Level::Debug,
        _ => log::Level::Trace,
    }
}

/// Handle `log` — forward a log message from the plugin.
pub(crate) fn handle_log(plugin_id: &PluginId, level: u32, msg: &str) {
    log::log!(
        target: "plugin",
        log_level_from_u32(level),
        "[{plugin_id}]: {msg}"
    );
}

/// Read a stream to its end, keeping at most `limit` bytes. Reads on past the limit: stopping
/// early would leave the child blocked on a pipe nobody drains.
async fn read_capped<R>(mut reader: R, limit: usize) -> std::io::Result<(Vec<u8>, bool)>
where
    R: AsyncRead + Unpin,
{
    let mut collected = Vec::new();
    // Heap-allocated on purpose: two of these live across an await in `run_capped`, and a
    // stack array would put 16 KiB into every `run_command` future.
    let mut chunk = vec![0_u8; 8192];
    let mut truncated = false;

    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }

        let free = limit.saturating_sub(collected.len());
        let taken = free.min(read);
        collected.extend_from_slice(&chunk[..taken]);
        truncated |= taken < read;
    }

    Ok((collected, truncated))
}

/// What a capped run produced, and which of its streams the cap cut short.
#[derive(Debug)]
struct CappedOutput {
    output: std::process::Output,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

/// A started command together with every process it starts: a job object on Windows, a process
/// group on Unix. Unless the run is [finished](Self::finish), dropping it kills the whole tree,
/// so a run that hits its time limit, or whose call is cancelled, leaves no grandchild behind. A
/// process that leaves the group on purpose (`setsid`) still escapes on Unix: `run_command`
/// narrows, it does not isolate.
#[derive(Debug)]
struct ProcessTree {
    child: Box<dyn ChildWrapper>,
    finished: bool,
}

impl ProcessTree {
    fn spawn(command: &SerializableCommand) -> crate::Result<Self> {
        let mut cmd = command.to_tokio_command();
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut wrapped = CommandWrap::from(cmd);
        #[cfg(windows)]
        wrapped.wrap(process_wrap::tokio::JobObject);
        #[cfg(unix)]
        wrapped.wrap(process_wrap::tokio::ProcessGroup::leader());
        // A safety net on Windows too: the job dies with its last handle, even if the launcher does.
        wrapped.wrap(process_wrap::tokio::KillOnDrop);

        let child = wrapped.spawn().map_err(|err| {
            crate::ErrorKind::CoreError(format!("Failed to run `{}`: {err}", command.program))
                .as_error()
        })?;

        Ok(Self {
            child,
            finished: false,
        })
    }

    /// Wait for the command to exit. On Unix that is the group's leader alone: it can be gone
    /// while a grandchild that inherited its output still runs, so this is not the run's end.
    async fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait().await
    }

    /// The run is over, its output read to the end: nothing is left to stop on drop.
    fn finish(&mut self) {
        self.finished = true;
    }
}

impl Drop for ProcessTree {
    fn drop(&mut self) {
        // An unfinished run is stopped as a whole, also when its leader has exited already
        // (a grandchild may be holding the output open). The group is signalled
        // unconditionally: while a member lives its id cannot be taken by another process, and
        // when none does the signal finds no group and the error is ignored.
        if !self.finished {
            let _ = self.child.start_kill();
        }
    }
}

/// Run an already-validated command with both caps applied. The caps are parameters only for
/// tests; production passes [`COMMAND_TIMEOUT`] and `PluginSandboxLimits::max_output_bytes`.
async fn run_capped(
    command: &SerializableCommand,
    timeout: Duration,
    max_output_bytes: usize,
) -> crate::Result<CappedOutput> {
    // On timeout or cancellation `tree` is dropped unwaited, which kills all of it.
    let mut tree = ProcessTree::spawn(command)?;
    collect_capped(&mut tree, command, timeout, max_output_bytes).await
}

/// Read what `tree` prints and wait for it, within `timeout`.
async fn collect_capped(
    tree: &mut ProcessTree,
    command: &SerializableCommand,
    timeout: Duration,
    max_output_bytes: usize,
) -> crate::Result<CappedOutput> {
    let stdout = tree.child.stdout().take().ok_or_else(|| {
        crate::ErrorKind::CoreError("Failed to capture command stdout".to_owned()).as_error()
    })?;
    let stderr = tree.child.stderr().take().ok_or_else(|| {
        crate::ErrorKind::CoreError("Failed to capture command stderr".to_owned()).as_error()
    })?;

    let collect = async {
        tokio::try_join!(
            read_capped(stdout, max_output_bytes),
            read_capped(stderr, max_output_bytes),
            tree.wait(),
        )
    };

    match tokio::time::timeout(timeout, collect).await {
        Ok(Ok(((stdout, stdout_truncated), (stderr, stderr_truncated), status))) => {
            tree.finish();
            Ok(CappedOutput {
                output: std::process::Output {
                    status,
                    stdout,
                    stderr,
                },
                stdout_truncated,
                stderr_truncated,
            })
        }
        Ok(Err(err)) => Err(crate::ErrorKind::CoreError(format!(
            "Failed to read output of `{}`: {err}",
            command.program
        ))
        .as_error()),
        Err(_elapsed) => Err(crate::ErrorKind::CoreError(format!(
            "Command `{}` exceeded the {} second time limit and was killed",
            command.program,
            timeout.as_secs()
        ))
        .as_error()),
    }
}

/// Handle `run_command` — execute a command on behalf of a plugin.
pub(crate) async fn handle_run_command(
    plugin_id: &PluginId,
    command: CommandDto,
    container: &AetherContainer,
) -> crate::Result<OutputDto> {
    let command_for_log = command.clone();
    log::debug!(target: "plugin", "[{plugin_id}]: run_command {command_for_log:?}");

    // The allowlist is whatever the core hands out through `get_java` / `install_java`, so a
    // system-wide Java the user registered counts too. Read on every call: a runtime removed
    // since must stop being allowed.
    let registered_java = container.list_java_use_case().execute().await?;
    let allowed_program_dirs = plugin_utils::java_bin_dirs(registered_java.iter().map(Java::path));

    let host_command = plugin_utils::plugin_command_to_host(
        plugin_id,
        &command,
        &container.location_info(),
        &allowed_program_dirs,
    )?;

    log::debug!(target: "plugin", "[{plugin_id}]: running {host_command:?}");
    let max_output_bytes = container.sandbox_limits().max_output_bytes;
    let CappedOutput {
        output,
        stdout_truncated,
        stderr_truncated,
    } = run_capped(&host_command, COMMAND_TIMEOUT, max_output_bytes).await?;

    if stdout_truncated || stderr_truncated {
        log::warn!(
            target: "plugin",
            "[{plugin_id}]: output of `{}` was truncated at {max_output_bytes} bytes per stream \
             (stdout: {stdout_truncated}, stderr: {stderr_truncated})",
            host_command.program
        );
    }

    if !output.status.success() {
        log::error!(
            target: "plugin",
            "[{plugin_id}]: command failed: {:?}, stderr: {:?}",
            command_for_log,
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(crate::ErrorKind::CoreError("Command execution failed".to_string()).as_error());
    }

    Ok(OutputDto::from_output(
        &output,
        stdout_truncated,
        stderr_truncated,
    ))
}

// ── Extism host function wrappers ──

host_fn!(
pub log(user_data: PluginContext; level: u32, msg: String) -> () {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();

    handle_log(&id, level, &msg);
    Ok(())
});

host_fn!(
pub get_id(user_data: PluginContext;) -> String {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.to_string();

    Ok(id)
});

host_fn!(
pub run_command(user_data: PluginContext; command: Msgpack<CommandDto>) -> HostResult<OutputDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<OutputDto>(
        block_on(handle_run_command(&id, command.0, &container))
    )
});

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn max_output_bytes() -> usize {
        crate::shared::limits::PluginSandboxLimits::default().max_output_bytes
    }

    // ── read_capped ──

    #[rstest]
    #[case::shorter_than_the_limit(16)]
    #[case::exactly_at_the_limit(5)]
    #[tokio::test]
    async fn should_return_output_untouched_when_it_fits_the_limit(#[case] limit: usize) {
        let (collected, truncated) = read_capped(&b"hello"[..], limit).await.unwrap();

        assert_eq!(collected, b"hello");
        assert!(!truncated);
    }

    #[tokio::test]
    async fn should_truncate_output_when_it_is_longer_than_the_limit() {
        // Far more than one read chunk past the limit, so the loop has to keep draining
        // instead of walking away and leaving a child blocked on a full pipe.
        let source = vec![b'x'; 128 * 1024];

        let (collected, truncated) = read_capped(&source[..], 1024).await.unwrap();

        assert!(truncated);
        assert_eq!(
            collected,
            &source[..1024],
            "exactly the first `limit` bytes, with nothing appended"
        );
    }

    // ── run_capped, against a real child process ──

    /// Run `script` through the platform shell.
    ///
    /// `run_capped` runs whatever it is handed: the allowlist that refuses a shell lives one
    /// layer up, in `plugin_command_to_host`, and is tested there.
    fn shell_command(script: &str) -> SerializableCommand {
        let (program, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_owned(), script.to_owned()])
        } else {
            ("sh", vec!["-c".to_owned(), script.to_owned()])
        };

        SerializableCommand {
            program: program.to_owned(),
            args,
            current_dir: None,
            env: None,
        }
    }

    /// Print one long line `repeat` times.
    fn noisy_script(repeat: u32) -> String {
        let line = "a".repeat(64);
        if cfg!(windows) {
            format!("for /L %i in (1,1,{repeat}) do @echo {line}")
        } else {
            format!("i=0; while [ $i -lt {repeat} ]; do echo {line}; i=$((i+1)); done")
        }
    }

    #[tokio::test]
    async fn should_cap_the_stdout_when_a_real_process_prints_too_much() {
        let captured = run_capped(&shell_command(&noisy_script(200)), COMMAND_TIMEOUT, 512)
            .await
            .expect("child should run");

        assert!(captured.output.status.success());
        assert!(
            captured.stdout_truncated,
            "13 KiB of stdout against a 512 byte cap"
        );
        assert!(!captured.stderr_truncated, "nothing was written to stderr");
        assert_eq!(captured.output.stdout.len(), 512);
    }

    #[tokio::test]
    async fn should_return_output_untruncated_when_it_is_under_the_cap() {
        let captured = run_capped(
            &shell_command("echo hello"),
            COMMAND_TIMEOUT,
            max_output_bytes(),
        )
        .await
        .expect("child should run");

        assert!(!captured.stdout_truncated && !captured.stderr_truncated);
        assert!(String::from_utf8_lossy(&captured.output.stdout).contains("hello"));
    }

    #[tokio::test]
    async fn should_kill_a_child_when_it_outlives_the_time_limit() {
        let script = if cfg!(windows) {
            "ping -n 60 127.0.0.1 > nul"
        } else {
            "sleep 60"
        };

        let err = run_capped(
            &shell_command(script),
            Duration::from_millis(200),
            max_output_bytes(),
        )
        .await
        .expect_err("a child past the time limit must be an error, not a wait");

        assert!(err.to_string().contains("time limit"), "{err}");
    }

    /// A shell that starts long-lived grandchildren of the test and waits on them.
    const SCRIPT_WITH_GRANDCHILDREN: &str = if cfg!(windows) {
        "start /b ping -n 600 127.0.0.1 > nul & ping -n 600 127.0.0.1 > nul"
    } else {
        "sleep 600 & sleep 600 & wait"
    };

    /// The grandchild program [`SCRIPT_WITH_GRANDCHILDREN`] starts.
    const GRANDCHILD_NAME: &str = if cfg!(windows) { "ping" } else { "sleep" };

    /// A live process, by pid and start time, so a reused pid is not mistaken for it.
    type ProcessKey = (sysinfo::Pid, u64);

    fn is_alive(process: &sysinfo::Process) -> bool {
        process.status() != sysinfo::ProcessStatus::Zombie
    }

    /// The live processes named `name` anywhere below `root`.
    fn descendants_named(system: &mut sysinfo::System, root: u32, name: &str) -> Vec<ProcessKey> {
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let mut found = Vec::new();
        let mut parents = vec![sysinfo::Pid::from_u32(root)];
        while let Some(parent) = parents.pop() {
            for (pid, process) in system.processes() {
                if process.parent() == Some(parent) && is_alive(process) {
                    parents.push(*pid);
                    // Windows names it `PING.EXE`.
                    let process_name = process.name().to_string_lossy().to_ascii_lowercase();
                    if process_name.starts_with(name) {
                        found.push((*pid, process.start_time()));
                    }
                }
            }
        }
        found
    }

    fn any_alive(system: &mut sysinfo::System, processes: &[ProcessKey]) -> bool {
        let pids: Vec<_> = processes.iter().map(|(pid, _)| *pid).collect();
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&pids), true);

        processes.iter().any(|(pid, started)| {
            system
                .process(*pid)
                .is_some_and(|process| process.start_time() == *started && is_alive(process))
        })
    }

    /// Poll until `done` holds. A condition polled, not a delay guessed: how long a process takes
    /// to start or to die is up to the OS, so only the give-up deadline is generous.
    async fn eventually(what: &str, mut done: impl FnMut() -> bool) {
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        while !done() {
            assert!(
                std::time::Instant::now() < deadline,
                "timed out waiting until {what}"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Start [`SCRIPT_WITH_GRANDCHILDREN`] and wait until its grandchildren are running.
    async fn tree_with_grandchildren(
        system: &mut sysinfo::System,
    ) -> (SerializableCommand, ProcessTree, Vec<ProcessKey>) {
        let command = shell_command(SCRIPT_WITH_GRANDCHILDREN);
        let tree = ProcessTree::spawn(&command).expect("child should run");
        let root = tree.child.id().expect("a running child has a pid");

        let mut grandchildren = Vec::new();
        eventually("the grandchildren are running", || {
            grandchildren = descendants_named(system, root, GRANDCHILD_NAME);
            !grandchildren.is_empty()
        })
        .await;

        (command, tree, grandchildren)
    }

    #[tokio::test]
    async fn should_kill_the_grandchildren_when_the_run_outlives_the_time_limit() {
        let mut system = sysinfo::System::new();
        let (command, mut tree, grandchildren) = tree_with_grandchildren(&mut system).await;

        let err = collect_capped(
            &mut tree,
            &command,
            Duration::from_millis(100),
            max_output_bytes(),
        )
        .await
        .expect_err("a tree past the time limit must be an error, not a wait");
        assert!(err.to_string().contains("time limit"), "{err}");
        // What `run_capped` does with it on that error.
        drop(tree);

        eventually("the grandchildren are dead", || {
            !any_alive(&mut system, &grandchildren)
        })
        .await;
    }

    /// A grandchild outliving its parent, the group's leader, keeps the output pipe open, so the
    /// run is not over when the leader exits: it has to end at the time limit with the grandchild
    /// killed, not left behind because the leader had been waited out.
    #[cfg(unix)]
    #[tokio::test]
    async fn should_kill_the_grandchild_when_the_leader_exited_and_the_run_outlives_the_time_limit()
    {
        let pid_file = tempfile::NamedTempFile::new().unwrap();
        let command = shell_command(&format!(
            "sleep 600 & echo $! > '{}'",
            pid_file.path().display()
        ));
        let mut tree = ProcessTree::spawn(&command).expect("child should run");

        let mut system = sysinfo::System::new();
        let mut grandchild = Vec::new();
        eventually("the grandchild is running", || {
            let pid = std::fs::read_to_string(pid_file.path())
                .ok()
                .and_then(|text| text.trim().parse::<u32>().ok())
                .map(sysinfo::Pid::from_u32);
            if let Some(pid) = pid {
                system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
                grandchild = system
                    .process(pid)
                    .map(|process| vec![(pid, process.start_time())])
                    .unwrap_or_default();
            }
            !grandchild.is_empty()
        })
        .await;

        let err = collect_capped(
            &mut tree,
            &command,
            Duration::from_millis(500),
            max_output_bytes(),
        )
        .await
        .expect_err("a grandchild holding the output past the time limit must be an error");
        assert!(err.to_string().contains("time limit"), "{err}");
        drop(tree);

        eventually("the grandchild is dead", || {
            !any_alive(&mut system, &grandchild)
        })
        .await;
    }

    #[tokio::test]
    async fn should_kill_the_grandchildren_when_the_run_is_cancelled() {
        let mut system = sysinfo::System::new();
        let (_, tree, grandchildren) = tree_with_grandchildren(&mut system).await;

        // A cancelled `run_command` drops its future, and the tree with it.
        drop(tree);

        eventually("the grandchildren are dead", || {
            !any_alive(&mut system, &grandchildren)
        })
        .await;
    }

    /// A variable the launcher really has and [`plugin_utils::isolated_env`] does not pass on.
    /// Chosen from the live environment rather than set here, so the test never mutates
    /// process-wide state other tests read. Panics when there is none, so the tests cannot pass
    /// vacuously.
    fn a_variable_that_must_not_leak() -> (String, String) {
        let isolated = plugin_utils::isolated_env();

        std::env::vars()
            .find(|(name, value)| {
                // Windows keeps hidden `=C:`-style entries that the shell rebuilds from its own
                // working directory; they are not inherited state, so skip anything not spellable
                // as a plain variable reference.
                name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !name.is_empty()
                && value.len() > 8
                && value.is_ascii()
                // Windows environment names are case-insensitive.
                && !isolated
                    .keys()
                    .any(|kept| kept.eq_ignore_ascii_case(name))
            })
            .expect("the test launcher must have an environment variable outside the isolated set")
    }

    #[tokio::test]
    async fn should_pass_only_the_explicit_environment_when_a_child_is_started() {
        let (name, value) = a_variable_that_must_not_leak();

        let script = if cfg!(windows) {
            format!("echo [%{name}%]")
        } else {
            format!("echo \"[${name}]\"")
        };

        let mut command = shell_command(&script);
        command.env = Some(plugin_utils::isolated_env());

        let captured = run_capped(&command, COMMAND_TIMEOUT, max_output_bytes())
            .await
            .expect("child should run");
        let stdout = String::from_utf8_lossy(&captured.output.stdout);

        assert!(
            !stdout.contains(&value),
            "`{name}` must not reach a plugin-started process, but the child printed: {stdout}"
        );
    }

    #[tokio::test]
    async fn should_still_inherit_the_environment_when_none_is_given() {
        let (name, value) = a_variable_that_must_not_leak();

        let script = if cfg!(windows) {
            format!("echo [%{name}%]")
        } else {
            format!("echo \"[${name}]\"")
        };

        // `env: None` is how user-configured pre-launch / post-exit commands still run.
        let captured = run_capped(&shell_command(&script), COMMAND_TIMEOUT, max_output_bytes())
            .await
            .expect("child should run");

        assert!(String::from_utf8_lossy(&captured.output.stdout).contains(&value));
    }
}
