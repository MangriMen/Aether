use std::{path::Path, process::Stdio, sync::Arc, time::Duration};

use async_trait::async_trait;
use process_wrap::tokio::{ChildWrapper, CommandWrap};
use tokio_util::sync::CancellationToken;

use crate::{
    features::{
        auth::Credential,
        instance::{
            Instance, InstanceError, InstanceId, InstanceInstallService, InstanceInstallStage,
            InstanceLaunchService, InstanceLocks, InstanceStorage, InstanceStorageExt,
            LaunchCancellations, LockRefused,
        },
        minecraft::{
            GetMinecraftLaunchCommandParams, LaunchSettings, MinecraftHealthParams,
            MinecraftHealthService, MinecraftLaunchCommandService,
        },
        plugins::{PluginInternalEvent, PluginRegistry, PluginState},
        process::{
            KillScope, MinecraftProcessMetadata, ProcessError, ProcessStartService, ProcessStorage,
        },
        settings::{DefaultInstanceSettings, DefaultInstanceSettingsStorage, Hooks, LocationInfo},
    },
    shared::{
        process_tree::wrap_tree,
        serializable_command::{
            HookKind, HookSpawnError, SerializableCommand, Wrapper, parse_hook_command,
            parse_wrapper,
        },
    },
};

/// How long a pre-launch hook may run. The instance stays locked meanwhile, and a hook that
/// never ends would keep the game from starting for good.
const PRELAUNCH_HOOK_TIMEOUT: Duration = Duration::from_mins(10);

pub struct LaunchInstanceUseCase {
    plugin_registry: Arc<PluginRegistry>,
    instance_storage: Arc<dyn InstanceStorage>,
    default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
    location_info: Arc<LocationInfo>,
    process_storage: Arc<dyn ProcessStorage>,
    instance_install_service: Arc<dyn InstanceInstallService>,
    minecraft_health_service: Arc<dyn MinecraftHealthService>,
    minecraft_launch_command_service: Arc<dyn MinecraftLaunchCommandService>,
    process_start_service: Arc<dyn ProcessStartService>,
    instance_locks: Arc<InstanceLocks>,
    cancellations: Arc<LaunchCancellations>,
}

impl LaunchInstanceUseCase {
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        instance_storage: Arc<dyn InstanceStorage>,
        default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
        location_info: Arc<LocationInfo>,
        process_storage: Arc<dyn ProcessStorage>,
        instance_install_service: Arc<dyn InstanceInstallService>,
        minecraft_health_service: Arc<dyn MinecraftHealthService>,
        minecraft_launch_command_service: Arc<dyn MinecraftLaunchCommandService>,
        process_start_service: Arc<dyn ProcessStartService>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            plugin_registry,
            instance_storage,
            default_instance_settings_storage,
            location_info,
            process_storage,
            instance_install_service,
            minecraft_health_service,
            minecraft_launch_command_service,
            process_start_service,
            instance_locks,
            cancellations: Arc::default(),
        }
    }

    /// Registers the launches in `cancellations`, so that the user can call them off.
    #[must_use]
    pub fn with_cancellations(mut self, cancellations: Arc<LaunchCancellations>) -> Self {
        self.cancellations = cancellations;
        self
    }

    fn resolve_launch_settings(
        instance: &Instance,
        settings: &DefaultInstanceSettings,
    ) -> LaunchSettings {
        LaunchSettings {
            launch_args: instance.launch_args.resolve(settings.launch_args()),
            env_vars: instance.env_vars.resolve(settings.env_vars()),
            memory: instance.memory.resolve(settings.memory()),
            window: instance.window.resolve(settings.window()),
            hooks: instance.hooks.resolve(settings.hooks()),
        }
    }

    async fn notify_plugin_event(
        &self,
        instance: &Instance,
        event: PluginInternalEvent,
        event_name: &str,
    ) {
        // The handle is cloned out of the registry: holding the entry over the call would block
        // a disable or restart of the plugin, which writes it, on the runtime thread. A plugin
        // that is not loaded, or stops while it is called, only misses the event.
        let handle = instance.pack_info.as_ref().and_then(|pack_info| {
            let plugin = self
                .plugin_registry
                .get(&pack_info.provider_id.plugin_id)
                .ok()?;
            match &plugin.state {
                PluginState::Loaded(handle) => Some(handle.clone()),
                _ => None,
            }
        });
        if let Some(pack_info) = &instance.pack_info
            && let Some(handle) = handle
            && let Err(e) = handle
                .run("handle_event", move |plugin| plugin.handle_event(&event))
                .await
                .flatten()
        {
            tracing::warn!(
                "Plugin '{}' failed to handle {}: {}",
                pack_info.provider_id.plugin_id,
                event_name,
                e
            );
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn execute(
        &self,
        instance_id: InstanceId,
        credentials: Credential,
    ) -> Result<MinecraftProcessMetadata, InstanceError> {
        // Held until the game is started: content is not installed or updated under it. The
        // stage below does not tell an install into an `Installed` instance, nor one that starts
        // after it was read.
        let _lock = match self.instance_locks.try_lock(&instance_id) {
            Ok(lock) => lock,
            Err(LockRefused::Held) => {
                return Err(InstanceError::InstanceStillInstalling {
                    instance_id: instance_id.into(),
                });
            }
            Err(LockRefused::Paused) => return Err(InstanceError::CachesBeingCleared),
        };
        // After the lock, so that a launch refused for it never takes the place of the one that
        // holds it. Dropped before the lock.
        let launch = self.cancellations.register(&instance_id);
        let cancel = &launch.token.clone();
        let settings = self.default_instance_settings_storage.get().await?;
        let mut instance = self.instance_storage.get(&instance_id).await?;

        let launch_settings = Self::resolve_launch_settings(&instance, &settings);

        if instance.install_stage == InstanceInstallStage::PackInstalling
            || instance.install_stage == InstanceInstallStage::Installing
        {
            return Err(InstanceError::InstanceStillInstalling {
                instance_id: instance_id.into(),
            });
        }

        // Check if profile has a running profile, and reject running the command if it does
        // Done late so a quick double call doesn't launch two instances
        if let Some(process) = self
            .process_storage
            .list_metadata()
            .await?
            .into_iter()
            .find(|p| *p.instance_id() == instance_id)
        {
            return Err(InstanceError::InstanceAlreadyRunning {
                instance_id: instance_id.to_string(),
                process_id: process.uuid(),
            });
        }

        // Stat-only health check before launch (uses MinecraftHealthService).
        // Checks every library, client.jar, Java binary, and asset index.
        // On network error (offline), unwrap_or(true) skips the check.
        // If something is missing, the full install runs with a progress bar.
        ensure_not_cancelled(cancel)?;
        if instance.install_stage != InstanceInstallStage::Installed
            || !self
                .minecraft_health_service
                .verify_files(MinecraftHealthParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    java_path: instance.java_path_override(),
                })
                .await
                .unwrap_or(true)
        {
            self.instance_install_service
                .execute(instance_id.clone(), false, None)
                .await?;
            // The install may have pinned the loader version.
            instance = self.instance_storage.get(&instance_id).await?;
        }

        let instance_path = self.location_info.instance_dir(instance.id());

        let wrapper = run_pre_launch_hook(
            &launch_settings.hooks,
            &instance_path,
            PRELAUNCH_HOOK_TIMEOUT,
            cancel,
        )
        .await?;
        ensure_not_cancelled(cancel)?;

        self.notify_plugin_event(
            &instance,
            PluginInternalEvent::BeforeInstanceLaunch {
                instance_id: instance.id().clone(),
            },
            "BeforeInstanceLaunch",
        )
        .await;
        ensure_not_cancelled(cancel)?;

        let launch_command = self
            .minecraft_launch_command_service
            .execute(
                GetMinecraftLaunchCommandParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    launch_dir: instance_path,
                    java_path: instance.java_path_override(),
                    wrapper,
                },
                launch_settings.clone(),
                credentials,
            )
            .await?;
        // The last stop, and the launch leaves the registry with it: past it a call off finds no
        // launch, and the user stops the game.
        if !launch.commit() {
            return Err(InstanceError::Cancelled);
        }

        self.instance_storage
            .upsert_with(instance.id(), |instance| {
                instance.last_played = Some(chrono::Utc::now());
                Ok(())
            })
            .await?;

        // Behind a wrapper the started process is the wrapper: stopping it alone would leave
        // the game running.
        let behind_wrapper = launch_command.wrapped;
        let kill_scope = if behind_wrapper {
            KillScope::Tree
        } else {
            KillScope::Process
        };
        let metadata = self
            .process_start_service
            .execute(
                instance_id,
                launch_command.command,
                kill_scope,
                launch_settings.hooks.post_exit().to_owned(),
                launch_command.secrets,
            )
            .await
            .map_err(|err| start_error(err, behind_wrapper));

        self.notify_plugin_event(
            &instance,
            PluginInternalEvent::AfterInstanceLaunch {
                instance_id: instance.id().clone(),
            },
            "AfterInstanceLaunch",
        )
        .await;

        metadata
    }
}

#[async_trait]
impl InstanceLaunchService for LaunchInstanceUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
        credentials: Credential,
    ) -> Result<MinecraftProcessMetadata, InstanceError> {
        self.execute(instance_id, credentials).await
    }
}

/// Why the game did not start. Behind a wrapper, the program that could not be started is the
/// user's wrapper, not java.
fn start_error(err: ProcessError, wrapped: bool) -> InstanceError {
    match err {
        ProcessError::SpawnFailed { program, source } if wrapped => HookSpawnError::SpawnFailed {
            hook: HookKind::Wrapper,
            program,
            source,
        }
        .into(),
        other => other.into(),
    }
}

fn ensure_not_cancelled(cancel: &CancellationToken) -> Result<(), InstanceError> {
    if cancel.is_cancelled() {
        Err(InstanceError::Cancelled)
    } else {
        Ok(())
    }
}

/// Runs the pre-launch hook in `dir` and waits for it for `timeout`, and gives the wrapper the
/// game is to be started by. An empty hook is no hook. A hook still running at the deadline, or
/// when `cancel` fires, is killed with everything it started: the game does not start under it.
///
/// The wrapper is read first: a typo in it would only stop the launch after the pre-launch hook
/// had already done its work.
async fn run_pre_launch_hook(
    hooks: &Hooks,
    dir: &Path,
    timeout: Duration,
    cancel: &CancellationToken,
) -> Result<Option<Wrapper>, InstanceError> {
    let wrapper = parse_wrapper(hooks.wrapper(), dir)?;
    if let Some(command) = parse_hook_command(HookKind::PreLaunch, hooks.pre_launch(), dir)? {
        run_hook_command(&command, timeout, cancel).await?;
    }
    Ok(wrapper)
}

/// A started hook with every process it starts. Unless it [ended](Self::end), dropping it kills
/// the whole tree: the launch that waits for the hook may be dropped too. A hook that ends by
/// itself leaves what it started in the background alone, which is why neither the group nor the
/// job object kill their members when the hook is let go.
struct HookProcess {
    child: Box<dyn ChildWrapper>,
    ended: bool,
}

impl HookProcess {
    fn spawn(command: &SerializableCommand) -> std::io::Result<Self> {
        let mut command = command.to_tokio_command();
        // A hook that reads the terminal, in a group of its own, would be stopped by it and hang.
        command.stdin(Stdio::null());
        let mut command = CommandWrap::from(command);
        wrap_tree(&mut command);
        Ok(Self {
            child: command.spawn()?,
            ended: false,
        })
    }

    /// Waits for the hook process alone: the one that ends is the one the user's command started,
    /// and what it left behind is not waited for.
    async fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.inner_mut().wait().await
    }

    /// The hook ended by itself: nothing is left to stop on drop.
    fn end(&mut self) {
        self.ended = true;
    }

    /// Kills the hook with everything it started, and waits for the process to be gone.
    async fn kill(&mut self) {
        if let Err(err) = self.child.start_kill() {
            tracing::warn!("Failed to kill the pre-launch hook: {err}");
        }
        let _ = self.wait().await;
        self.ended = true;
    }
}

impl Drop for HookProcess {
    fn drop(&mut self) {
        if !self.ended {
            let _ = self.child.start_kill();
        }
    }
}

async fn run_hook_command(
    command: &SerializableCommand,
    timeout: Duration,
    cancel: &CancellationToken,
) -> Result<(), InstanceError> {
    // Not even started when it is called off already.
    ensure_not_cancelled(cancel)?;
    let mut hook = HookProcess::spawn(command)
        .map_err(|err| HookSpawnError::spawn_failed(HookKind::PreLaunch, &command.program, err))?;

    let status = tokio::select! {
        status = hook.wait() => status,
        () = tokio::time::sleep(timeout) => {
            hook.kill().await;
            return Err(InstanceError::PrelaunchCommandTimeout {
                seconds: timeout.as_secs(),
            });
        }
        () = cancel.cancelled() => {
            hook.kill().await;
            return Err(InstanceError::Cancelled);
        }
    };
    hook.end();
    let status = status.map_err(|err| HookSpawnError::WaitFailed {
        hook: HookKind::PreLaunch,
        program: command.program.clone(),
        source: Arc::new(err),
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(InstanceError::PrelaunchCommandError {
            code: status.code().unwrap_or(-1),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::serializable_command::CommandParseError;

    /// Not a test: the process the hook tests start (this very binary), which never ends.
    #[test]
    #[ignore = "started by the pre-launch hook tests as a process that hangs"]
    fn hang_as_a_hook() {
        std::thread::sleep(Duration::from_secs(600));
    }

    /// A token that nobody fires.
    fn never() -> CancellationToken {
        CancellationToken::new()
    }

    /// Not a test: a hook that starts a grandchild and writes its pid to the file named by
    /// `AETHER_HOOK_PID_FILE`. It then waits for the grandchild, or ends at once when
    /// `AETHER_HOOK_LEAVES` is set.
    #[test]
    #[ignore = "started by the pre-launch hook tests as a hook with a grandchild"]
    fn start_a_grandchild_as_a_hook() {
        let module = module_path!().trim_start_matches("aether_core::");
        let mut grandchild = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", &format!("{module}::hang_as_a_hook")])
            .spawn()
            .unwrap();
        std::fs::write(
            std::env::var("AETHER_HOOK_PID_FILE").unwrap(),
            grandchild.id().to_string(),
        )
        .unwrap();
        if std::env::var_os("AETHER_HOOK_LEAVES").is_none() {
            grandchild.wait().unwrap();
        }
        // Left running on purpose when the hook leaves: the test kills it.
        #[allow(clippy::zombie_processes)]
        drop(grandchild);
    }

    /// The hook of `start_a_grandchild_as_a_hook`, writing the grandchild's pid to `pid_file`.
    fn hook_with_grandchild(pid_file: &Path, leaves: bool) -> SerializableCommand {
        let module = module_path!().trim_start_matches("aether_core::");
        let mut env: std::collections::BTreeMap<String, String> = std::env::vars().collect();
        env.insert(
            "AETHER_HOOK_PID_FILE".to_owned(),
            pid_file.to_string_lossy().into_owned(),
        );
        if leaves {
            env.insert("AETHER_HOOK_LEAVES".to_owned(), "1".to_owned());
        }
        SerializableCommand {
            program: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            args: vec![
                "--ignored".to_owned(),
                "--exact".to_owned(),
                format!("{module}::start_a_grandchild_as_a_hook"),
            ],
            current_dir: None,
            env: Some(env),
        }
    }

    async fn grandchild_pid(pid_file: &Path) -> sysinfo::Pid {
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(pid) = std::fs::read_to_string(pid_file)
                .ok()
                .and_then(|pid| pid.trim().parse::<u32>().ok())
            {
                return sysinfo::Pid::from_u32(pid);
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the hook wrote no pid"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// A zombie is gone too: nothing may have reaped it yet.
    fn is_running(pid: sysinfo::Pid) -> bool {
        let mut system = sysinfo::System::new().expect("system information");
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        system
            .process(pid)
            .is_some_and(|process| process.status() != sysinfo::ProcessStatus::Zombie)
    }

    async fn wait_until_gone(pid: sysinfo::Pid) {
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        while is_running(pid) {
            assert!(
                std::time::Instant::now() < deadline,
                "process {pid} is still running"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    fn kill(pid: sysinfo::Pid) {
        let mut system = sysinfo::System::new().expect("system information");
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        if let Some(process) = system.process(pid) {
            process.kill();
        }
    }

    #[tokio::test]
    async fn should_kill_what_the_hook_started_when_the_launch_is_called_off() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("pid");
        let cancel = never();
        let hook = hook_with_grandchild(&pid_file, false);
        let run = tokio::spawn({
            let cancel = cancel.clone();
            async move { run_hook_command(&hook, Duration::from_mins(5), &cancel).await }
        });
        let grandchild = grandchild_pid(&pid_file).await;

        cancel.cancel();
        let result = run.await.unwrap();

        assert!(
            matches!(result, Err(InstanceError::Cancelled)),
            "{result:?}"
        );
        wait_until_gone(grandchild).await;
    }

    #[tokio::test]
    async fn should_kill_what_the_hook_started_when_it_times_out() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("pid");
        let hook = hook_with_grandchild(&pid_file, false);

        let result = run_hook_command(&hook, Duration::from_secs(8), &never()).await;

        assert!(
            matches!(result, Err(InstanceError::PrelaunchCommandTimeout { .. })),
            "{result:?}"
        );
        wait_until_gone(grandchild_pid(&pid_file).await).await;
    }

    #[tokio::test]
    async fn should_leave_what_the_hook_started_when_it_ends_by_itself() {
        let dir = tempfile::tempdir().unwrap();
        let pid_file = dir.path().join("pid");
        let hook = hook_with_grandchild(&pid_file, true);

        let result = run_hook_command(&hook, Duration::from_mins(5), &never()).await;

        let grandchild = grandchild_pid(&pid_file).await;
        let running = is_running(grandchild);
        kill(grandchild);
        assert!(result.is_ok(), "{result:?}");
        assert!(running, "the background process was killed with the hook");
    }

    #[tokio::test]
    async fn should_not_start_the_hook_when_the_launch_is_called_off_already() {
        let cancel = never();
        cancel.cancel();

        // Started, this hook would fail to: the error would not be the call off.
        let result = run_pre_launch_hook(
            &pre_launch("aether-no-such-program-0f3a"),
            Path::new("."),
            Duration::from_secs(1),
            &cancel,
        )
        .await;

        assert!(
            matches!(result, Err(InstanceError::Cancelled)),
            "{result:?}"
        );
    }

    fn pre_launch(command: &str) -> Hooks {
        Hooks::new(command.to_owned(), String::new(), String::new())
    }

    #[tokio::test]
    async fn should_not_run_the_pre_launch_hook_when_the_wrapper_is_invalid() {
        // Run, this hook would fail to start: the error would name it, not the wrapper.
        let hooks = Hooks::new(
            "aether-no-such-program-0f3a".to_owned(),
            "\"prime-run".to_owned(),
            String::new(),
        );

        let result =
            run_pre_launch_hook(&hooks, Path::new("."), Duration::from_secs(1), &never()).await;

        assert!(
            matches!(
                result,
                Err(InstanceError::Hook(HookSpawnError::Invalid {
                    hook: HookKind::Wrapper,
                    reason: CommandParseError::UnclosedQuote,
                }))
            ),
            "{result:?}"
        );
    }

    fn hanging_hook() -> SerializableCommand {
        let module = module_path!().trim_start_matches("aether_core::");
        SerializableCommand {
            program: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            args: vec![
                "--ignored".to_owned(),
                "--exact".to_owned(),
                format!("{module}::hang_as_a_hook"),
            ],
            current_dir: None,
            env: None,
        }
    }

    #[tokio::test]
    async fn should_skip_the_hook_when_it_is_empty_or_blank() {
        for command in [
            "", "   ", "	
",
        ] {
            let result = run_pre_launch_hook(
                &pre_launch(command),
                Path::new("."),
                Duration::from_secs(1),
                &never(),
            )
            .await;
            assert!(result.is_ok(), "{command:?}: {result:?}");
        }
    }

    #[tokio::test]
    async fn should_fail_when_the_hook_cannot_be_started() {
        let result = run_pre_launch_hook(
            &pre_launch("aether-no-such-program-0f3a"),
            Path::new("."),
            Duration::from_secs(1),
            &never(),
        )
        .await;

        assert!(
            matches!(
                &result,
                Err(InstanceError::Hook(HookSpawnError::SpawnFailed {
                    hook: HookKind::PreLaunch,
                    program,
                    ..
                })) if program == "aether-no-such-program-0f3a"
            ),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn should_fail_without_running_anything_when_a_quote_of_the_hook_is_not_closed() {
        let result = run_pre_launch_hook(
            &pre_launch("\"aether-no-such-program-0f3a --token secret"),
            Path::new("."),
            Duration::from_secs(1),
            &never(),
        )
        .await;

        assert!(
            matches!(
                result,
                Err(InstanceError::Hook(HookSpawnError::Invalid {
                    hook: HookKind::PreLaunch,
                    reason: CommandParseError::UnclosedQuote,
                }))
            ),
            "{result:?}"
        );
    }

    fn spawn_failed() -> ProcessError {
        ProcessError::SpawnFailed {
            program: "gamemoderun".to_owned(),
            source: Arc::new(std::io::Error::from(std::io::ErrorKind::NotFound)),
        }
    }

    #[test]
    fn should_report_the_wrapper_when_the_wrapped_game_cannot_be_started() {
        let err = start_error(spawn_failed(), true);

        assert!(
            matches!(
                &err,
                InstanceError::Hook(HookSpawnError::SpawnFailed {
                    hook: HookKind::Wrapper,
                    program,
                    ..
                }) if program == "gamemoderun"
            ),
            "{err:?}"
        );
    }

    #[test]
    fn should_report_the_process_when_java_itself_cannot_be_started() {
        let err = start_error(spawn_failed(), false);

        assert!(
            matches!(
                err,
                InstanceError::ProcessError(ProcessError::SpawnFailed { .. })
            ),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn should_time_out_when_the_hook_does_not_end() {
        let started = std::time::Instant::now();

        let result = run_hook_command(&hanging_hook(), Duration::from_millis(500), &never()).await;

        assert!(
            matches!(result, Err(InstanceError::PrelaunchCommandTimeout { .. })),
            "{result:?}"
        );
        assert!(started.elapsed() < Duration::from_secs(30));
    }
}
