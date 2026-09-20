//! `log`, `get_id` and `run_command` — the core host functions exposed to plugins.
//!
//! `run_command` is the one place where a plugin can start a process on the host, so it is
//! fenced in on four sides (T-0.5):
//!
//! * only a program inside the core-managed Java tree may be started, by canonicalised path
//!   (the allowlist lives in [`plugin_utils::plugin_command_to_host`]);
//! * the working directory is mandatory and must sit inside a directory mounted for the plugin;
//! * the child gets an explicit minimal environment rather than the launcher's own;
//! * the run is capped in both wall-clock time and captured output.

use std::{process::Stdio, time::Duration};

use aether_core_plugin_api::v0::{CommandDto, OutputDto};
use extism::host_fn;
use extism_convert::Msgpack;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    core::app::AetherContainer,
    shared::{
        execute_async::infra::execute_async, serializable_command::domain::SerializableCommand,
    },
};

use super::super::{
    super::{
        super::plugin_utils,
        mappers::{OutputDtoExt, to_extism_res},
    },
    PluginContext,
};

/// Hard cap on how much of each stream (stdout, stderr) a plugin gets back. Everything past it
/// is read and dropped, so the child never blocks on a full pipe.
pub(crate) const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

/// Appended to a stream that hit [`MAX_OUTPUT_BYTES`], so the truncation is visible to whoever
/// reads the output and not only in the host log.
///
/// A flag on `OutputDto` would be the cleaner signal, but the plugin ABI encodes that DTO as a
/// fixed-length msgpack array: a fourth field makes every already-built plugin fail to decode
/// the response (see F-21).
pub(crate) const TRUNCATION_MARKER: &[u8] = b"\n[aether: output truncated]\n";

/// Wall-clock budget for one `run_command`. The packwiz installer downloads a whole modpack
/// within this, so it is deliberately generous: its job is to stop a wedged child, not to bound
/// useful work.
pub(crate) const COMMAND_TIMEOUT: Duration = Duration::from_mins(15);

// ── Testable business logic ──

/// Handle `log` — forward a log message from the plugin.
pub(crate) fn handle_log(plugin_id: &str, level: u32, msg: &str) {
    log::log!(
        target: "plugin",
        plugin_utils::log_level_from_u32(level),
        "[{plugin_id}]: {msg}"
    );
}

/// Read a stream to its end, keeping at most `limit` bytes.
///
/// Reading continues past the limit — the bytes are simply thrown away — because stopping
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

    if truncated {
        collected.extend_from_slice(TRUNCATION_MARKER);
    }

    Ok((collected, truncated))
}

/// Run an already-validated command with both caps applied, and report whether either stream
/// was truncated.
///
/// The caps are parameters rather than constants so tests can exercise the real code path
/// without waiting fifteen minutes or producing a megabyte; production always passes
/// [`COMMAND_TIMEOUT`] and [`MAX_OUTPUT_BYTES`].
async fn run_capped(
    command: &SerializableCommand,
    timeout: Duration,
    max_output_bytes: usize,
) -> crate::Result<(std::process::Output, bool)> {
    let mut cmd = command.to_tokio_command();
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // On timeout the future below is dropped; this is what turns that into a dead child.
        .kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|err| {
        crate::ErrorKind::CoreError(format!("Failed to run `{}`: {err}", command.program))
            .as_error()
    })?;

    let stdout = child.stdout.take().ok_or_else(|| {
        crate::ErrorKind::CoreError("Failed to capture command stdout".to_owned()).as_error()
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        crate::ErrorKind::CoreError("Failed to capture command stderr".to_owned()).as_error()
    })?;

    let collect = async move {
        tokio::try_join!(
            read_capped(stdout, max_output_bytes),
            read_capped(stderr, max_output_bytes),
            child.wait(),
        )
    };

    match tokio::time::timeout(timeout, collect).await {
        Ok(Ok(((stdout, stdout_truncated), (stderr, stderr_truncated), status))) => Ok((
            std::process::Output {
                status,
                stdout,
                stderr,
            },
            stdout_truncated || stderr_truncated,
        )),
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
    plugin_id: &str,
    command: CommandDto,
    container: &AetherContainer,
) -> crate::Result<OutputDto> {
    let command_for_log = command.clone();
    log::debug!(target: "plugin", "[{plugin_id}]: run_command {command_for_log:?}");

    let host_command =
        plugin_utils::plugin_command_to_host(plugin_id, &command, &container.location_info())?;

    log::debug!(target: "plugin", "[{plugin_id}]: running {host_command:?}");
    let (output, truncated) = run_capped(&host_command, COMMAND_TIMEOUT, MAX_OUTPUT_BYTES).await?;

    if truncated {
        log::warn!(
            target: "plugin",
            "[{plugin_id}]: output of `{}` was truncated at {MAX_OUTPUT_BYTES} bytes per stream",
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

    Ok(OutputDto::from_output(&output))
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
    let id = ctx.id.clone();

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
        execute_async(handle_run_command(&id, command.0, &container))
    )
});

#[cfg(test)]
mod tests {
    use super::*;

    // ── read_capped ──

    #[tokio::test]
    async fn should_return_output_shorter_than_the_limit_untouched() {
        let (collected, truncated) = read_capped(&b"hello"[..], 16).await.unwrap();

        assert_eq!(collected, b"hello");
        assert!(!truncated);
    }

    #[tokio::test]
    async fn should_return_output_exactly_at_the_limit_untouched() {
        let (collected, truncated) = read_capped(&b"hello"[..], 5).await.unwrap();

        assert_eq!(collected, b"hello");
        assert!(!truncated);
    }

    #[tokio::test]
    async fn should_truncate_output_longer_than_the_limit() {
        // Far more than one read chunk past the limit, so the loop has to keep draining
        // instead of walking away and leaving a child blocked on a full pipe.
        let source = vec![b'x'; 128 * 1024];

        let (collected, truncated) = read_capped(&source[..], 1024).await.unwrap();

        assert!(truncated);
        assert_eq!(
            collected.len(),
            1024 + TRUNCATION_MARKER.len(),
            "the kept bytes are capped; only the marker is added on top"
        );
        assert_eq!(&collected[..1024], &source[..1024]);
        assert!(collected.ends_with(TRUNCATION_MARKER));
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
    async fn should_cap_the_stdout_of_a_real_process() {
        let (output, truncated) =
            run_capped(&shell_command(&noisy_script(200)), COMMAND_TIMEOUT, 512)
                .await
                .expect("child should run");

        assert!(output.status.success());
        assert!(truncated, "13 KiB of stdout against a 512 byte cap");
        assert_eq!(output.stdout.len(), 512 + TRUNCATION_MARKER.len());
        assert!(output.stdout.ends_with(TRUNCATION_MARKER));
    }

    #[tokio::test]
    async fn should_return_untruncated_output_under_the_cap() {
        let (output, truncated) = run_capped(
            &shell_command("echo hello"),
            COMMAND_TIMEOUT,
            MAX_OUTPUT_BYTES,
        )
        .await
        .expect("child should run");

        assert!(!truncated);
        assert!(String::from_utf8_lossy(&output.stdout).contains("hello"));
    }

    #[tokio::test]
    async fn should_kill_a_child_that_outlives_the_time_limit() {
        let script = if cfg!(windows) {
            "ping -n 60 127.0.0.1 > nul"
        } else {
            "sleep 60"
        };

        let err = run_capped(
            &shell_command(script),
            Duration::from_millis(200),
            MAX_OUTPUT_BYTES,
        )
        .await
        .expect_err("a child past the time limit must be an error, not a wait");

        assert!(err.to_string().contains("time limit"), "{err}");
    }

    /// A variable the launcher really has and [`plugin_utils::isolated_env`] does not pass on.
    /// Chosen from the live environment rather than set here, so the test never mutates
    /// process-wide state other tests read.
    fn a_variable_that_must_not_leak() -> Option<(String, String)> {
        let isolated = plugin_utils::isolated_env();

        std::env::vars().find(|(name, value)| {
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
    }

    #[tokio::test]
    async fn should_pass_only_the_explicit_environment_to_the_child() {
        let Some((name, value)) = a_variable_that_must_not_leak() else {
            // Nothing to prove: the launcher's environment is already the minimal set.
            return;
        };

        let script = if cfg!(windows) {
            format!("echo [%{name}%]")
        } else {
            format!("echo \"[${name}]\"")
        };

        let mut command = shell_command(&script);
        command.env = Some(plugin_utils::isolated_env());

        let (output, _) = run_capped(&command, COMMAND_TIMEOUT, MAX_OUTPUT_BYTES)
            .await
            .expect("child should run");
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !stdout.contains(&value),
            "`{name}` must not reach a plugin-started process, but the child printed: {stdout}"
        );
    }

    #[tokio::test]
    async fn should_still_inherit_the_environment_when_none_is_given() {
        let Some((name, value)) = a_variable_that_must_not_leak() else {
            return;
        };

        let script = if cfg!(windows) {
            format!("echo [%{name}%]")
        } else {
            format!("echo \"[${name}]\"")
        };

        // `env: None` is how user-configured pre-launch / post-exit commands still run.
        let (output, _) = run_capped(&shell_command(&script), COMMAND_TIMEOUT, MAX_OUTPUT_BYTES)
            .await
            .expect("child should run");

        assert!(String::from_utf8_lossy(&output.stdout).contains(&value));
    }
}
