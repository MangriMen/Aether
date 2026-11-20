use std::io;
#[cfg(any(unix, test))]
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tracing::warn;

use crate::features::process::{
    AdoptedProcess, KillScope, OsProcessIdentity, ProcessProbe, ProcessRecord, ProcessState,
};

/// Reads process identities from the OS: `/proc` on Linux, where the start is kept in raw clock
/// ticks since boot, and `sysinfo` elsewhere, which gives the creation time in seconds.
pub struct OsProcessProbe {
    own: Option<OsProcessIdentity>,
}

impl OsProcessProbe {
    /// Reads the identity of the launcher once: it does not change while the launcher runs.
    pub fn new() -> Self {
        let own = read_identity(std::process::id())
            .inspect_err(|err| {
                warn!(
                    error = %err,
                    "Could not read the launcher's own process start: started games are not saved"
                );
            })
            .ok();
        Self { own }
    }
}

impl Default for OsProcessProbe {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProcessProbe for OsProcessProbe {
    async fn identity_of(&self, pid: u32) -> io::Result<OsProcessIdentity> {
        tokio::task::spawn_blocking(move || read_identity(pid))
            .await
            .map_err(io::Error::other)?
    }

    fn own_identity(&self) -> Option<&OsProcessIdentity> {
        self.own.as_ref()
    }

    async fn adopt(&self, record: &ProcessRecord) -> io::Result<Option<Box<dyn AdoptedProcess>>> {
        let identity = record.process.clone();
        let kill_scope = record.kill_scope;
        tokio::task::spawn_blocking(move || adopt(&identity, kill_scope))
            .await
            .map_err(io::Error::other)?
    }

    async fn is_alive(&self, record: &ProcessRecord) -> io::Result<bool> {
        let Some(mut process) = self.adopt(record).await? else {
            return Ok(false);
        };
        let state = tokio::task::spawn_blocking(move || process.try_wait())
            .await
            .map_err(io::Error::other)??;
        Ok(state == ProcessState::Running)
    }
}

#[cfg(windows)]
fn adopt(
    identity: &OsProcessIdentity,
    kill_scope: KillScope,
) -> io::Result<Option<Box<dyn AdoptedProcess>>> {
    Ok(
        super::windows_adopted::WindowsAdoptedProcess::open(identity, kill_scope)?
            .map(|process| Box::new(process) as Box<dyn AdoptedProcess>),
    )
}

#[cfg(unix)]
fn adopt(
    identity: &OsProcessIdentity,
    kill_scope: KillScope,
) -> io::Result<Option<Box<dyn AdoptedProcess>>> {
    if liveness(identity)? != Liveness::Running {
        return Ok(None);
    }
    let process: Box<dyn AdoptedProcess> = Box::new(UnixAdoptedProcess {
        identity: identity.clone(),
        kill_scope,
        looks: Looks::new(),
    });
    Ok(Some(process))
}

/// How long a look at an adopted process holds: the game is polled every 50 ms, and each look
/// reads the process table.
#[cfg(any(unix, test))]
const LIVENESS_TTL: Duration = Duration::from_secs(1);

/// How many looks in a row may fail before the failure is passed on. One failed look, such as
/// a moment out of file descriptors, must not end the following of a game that still runs.
#[cfg(any(unix, test))]
const MAX_FAILED_LOOKS: u32 = 3;

#[cfg(any(unix, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Liveness {
    Running,
    /// Ended, but not reaped by its parent yet: it still holds its pid and process group.
    Zombie,
    /// Gone, or the pid belongs to another process now.
    Gone,
}

/// What was last seen of an adopted process.
#[cfg(any(unix, test))]
#[derive(Debug)]
struct Looks {
    checked_at: Instant,
    exited: bool,
    failures: u32,
}

#[cfg(any(unix, test))]
impl Looks {
    /// Starts from a look that saw the process running.
    fn new() -> Self {
        Self {
            checked_at: Instant::now(),
            exited: false,
            failures: 0,
        }
    }

    fn is_due(&self) -> bool {
        !self.exited && self.checked_at.elapsed() >= LIVENESS_TTL
    }

    /// Takes in a look. A failed one keeps what was seen before and is tried again after the
    /// TTL; only [`MAX_FAILED_LOOKS`] failures in a row are passed on.
    fn record(&mut self, look: io::Result<Liveness>) -> io::Result<()> {
        self.checked_at = Instant::now();
        match look {
            Ok(liveness) => {
                self.failures = 0;
                self.exited = liveness != Liveness::Running;
                Ok(())
            }
            Err(err) => {
                self.failures += 1;
                if self.failures >= MAX_FAILED_LOOKS {
                    Err(err)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn state(&self) -> ProcessState {
        if self.exited {
            ProcessState::Exited(None)
        } else {
            ProcessState::Running
        }
    }
}

/// The pid to signal to stop `pid`, refused when the signal would reach the launcher: pid 0
/// and pids that do not fit `pid_t` name process groups or nothing, and the launcher must not
/// stop itself or, for a tree, its own group.
#[cfg(any(unix, test))]
fn kill_target(pid: u32, kill_scope: KillScope, own_pid: u32, own_group: u32) -> io::Result<i32> {
    let refuse = |why: &str| -> io::Result<i32> {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("refusing to stop process {pid}: {why}"),
        ))
    };
    let target = match i32::try_from(pid) {
        Ok(target) if target > 0 => target,
        _ => return refuse("not a process id"),
    };
    if pid == own_pid {
        return refuse("it is the launcher");
    }
    if kill_scope == KillScope::Tree && pid == own_group {
        return refuse("its group holds the launcher");
    }
    Ok(target)
}

/// A game an earlier launcher started. Only its parent may wait for it, so its exit is seen as
/// the process going away, and how it ended stays unknown.
#[cfg(unix)]
#[derive(Debug)]
struct UnixAdoptedProcess {
    identity: OsProcessIdentity,
    kill_scope: KillScope,
    looks: Looks,
}

#[cfg(unix)]
impl AdoptedProcess for UnixAdoptedProcess {
    fn try_wait(&mut self) -> io::Result<ProcessState> {
        if self.looks.is_due() {
            let look = liveness(&self.identity);
            if let Err(err) = &look {
                warn!(
                    pid = self.identity.pid(),
                    error = %err,
                    "Could not look at the adopted game process"
                );
            }
            self.looks.record(look)?;
        }
        Ok(self.looks.state())
    }

    fn start_kill(&mut self) -> io::Result<()> {
        use nix::errno::Errno;
        use nix::sys::signal::{Signal, kill, killpg};
        use nix::unistd::{Pid, getpgrp};

        let target = kill_target(
            self.identity.pid(),
            self.kill_scope,
            std::process::id(),
            getpgrp().as_raw().cast_unsigned(),
        )?;
        let pid = Pid::from_raw(target);
        // Looked at right before the signal: the pid of a game that has ended may belong to
        // another process by now. A zombie leader still holds its group, whose members may run.
        let sent = match (liveness(&self.identity)?, self.kill_scope) {
            (Liveness::Gone, _) | (Liveness::Zombie, KillScope::Process) => return Ok(()),
            (Liveness::Running | Liveness::Zombie, KillScope::Tree) => killpg(pid, Signal::SIGKILL),
            (Liveness::Running, KillScope::Process) => kill(pid, Signal::SIGKILL),
        };
        match sent {
            // It ended in between.
            Ok(()) | Err(Errno::ESRCH) => Ok(()),
            Err(errno) => Err(errno.into()),
        }
    }
}

#[cfg(target_os = "linux")]
fn read_identity(pid: u32) -> io::Result<OsProcessIdentity> {
    read_proc_stat(pid)?
        .map(|(identity, _)| identity)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("no process {pid}")))
}

/// The identity of `pid` and its state letter, from `/proc/<pid>/stat`; `None` when there is
/// no such process.
#[cfg(target_os = "linux")]
fn read_proc_stat(pid: u32) -> io::Result<Option<(OsProcessIdentity, char)>> {
    let path = format!("/proc/{pid}/stat");
    let stat = match std::fs::read_to_string(&path) {
        Ok(stat) => stat,
        // ESRCH: the process went away while its file was read.
        Err(err)
            if err.kind() == io::ErrorKind::NotFound
                || err.raw_os_error() == Some(nix::errno::Errno::ESRCH as i32) =>
        {
            return Ok(None);
        }
        Err(err) => return Err(err),
    };
    let invalid =
        |what: &str| io::Error::new(io::ErrorKind::InvalidData, format!("no {what} in {path}"));
    let start_ticks = parse_start_ticks(&stat).ok_or_else(|| invalid("start time"))?;
    let state = parse_state(&stat).ok_or_else(|| invalid("state"))?;
    let boot_id = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?;
    let identity = OsProcessIdentity::new(pid, start_ticks, boot_id.trim().to_owned());
    Ok(Some((identity, state)))
}

#[cfg(target_os = "linux")]
fn liveness(identity: &OsProcessIdentity) -> io::Result<Liveness> {
    let Some((current, state)) = read_proc_stat(identity.pid())? else {
        return Ok(Liveness::Gone);
    };
    Ok(if !identity.matches(&current) {
        Liveness::Gone
    } else if matches!(state, 'Z' | 'X') {
        Liveness::Zombie
    } else {
        Liveness::Running
    })
}

/// The `starttime` field (the 22nd) of a `/proc/<pid>/stat` line.
#[cfg(any(target_os = "linux", test))]
fn parse_start_ticks(stat: &str) -> Option<u64> {
    // The name in parentheses may hold spaces and parentheses; the fields after the last `)`
    // do not. The first of them is the 3rd field.
    stat[stat.rfind(')')? + 1..]
        .split_whitespace()
        .nth(22 - 3)?
        .parse()
        .ok()
}

/// The `state` field (the 3rd) of a `/proc/<pid>/stat` line.
#[cfg(any(target_os = "linux", test))]
fn parse_state(stat: &str) -> Option<char> {
    stat[stat.rfind(')')? + 1..]
        .split_whitespace()
        .next()?
        .chars()
        .next()
}

#[cfg(not(target_os = "linux"))]
fn read_identity(pid: u32) -> io::Result<OsProcessIdentity> {
    read_process(pid)?
        .map(|(identity, _)| identity)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("no process {pid}")))
}

/// The identity of `pid` and whether it is a zombie; `None` when there is no such process.
#[cfg(not(target_os = "linux"))]
fn read_process(pid: u32) -> io::Result<Option<(OsProcessIdentity, bool)>> {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, System};

    let mut system = System::new().map_err(io::Error::other)?;
    let target = Pid::from_u32(pid);
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[target]),
        false,
        ProcessRefreshKind::nothing(),
    );
    let Some(process) = system.process(target) else {
        return Ok(None);
    };
    // `sysinfo` reports 0 for a process it could not open: one is there, its start unknown.
    if process.start_time() == 0 {
        return Err(io::Error::other(format!(
            "the start time of process {pid} could not be read"
        )));
    }
    let identity = OsProcessIdentity::new(pid, process.start_time(), String::new());
    Ok(Some((identity, process.status() == ProcessStatus::Zombie)))
}

#[cfg(all(unix, not(target_os = "linux")))]
fn liveness(identity: &OsProcessIdentity) -> io::Result<Liveness> {
    let Some((current, zombie)) = read_process(identity.pid())? else {
        return Ok(Liveness::Gone);
    };
    Ok(if !identity.matches(&current) {
        Liveness::Gone
    } else if zombie {
        Liveness::Zombie
    } else {
        Liveness::Running
    })
}

#[cfg(test)]
mod tests {
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::features::instance::InstanceId;

    #[test]
    fn should_read_the_start_ticks_when_the_process_name_holds_spaces_and_parentheses() {
        let stat = "4242 (java (main) x) S 1 4242 4242 0 -1 4194560 100 0 0 0 5 2 0 0 20 0 \
                    30 0 987654 123456789 1000 18446744073709551615";

        assert_eq!(parse_start_ticks(stat), Some(987_654));
        assert_eq!(parse_state(stat), Some('S'));
    }

    #[test]
    fn should_read_no_start_ticks_when_the_line_is_cut_short() {
        assert_eq!(parse_start_ticks("4242 (java) S 1 4242"), None);
    }

    #[test]
    fn should_read_the_zombie_state_when_the_process_has_ended() {
        assert_eq!(parse_state("4242 (java) Z 1 4242"), Some('Z'));
    }

    fn failed_look() -> io::Result<Liveness> {
        Err(io::Error::other("too many open files"))
    }

    #[test]
    fn should_wait_a_while_before_the_next_look_when_the_process_was_just_seen() {
        assert!(!Looks::new().is_due());
    }

    #[test]
    fn should_keep_the_game_running_when_fewer_looks_than_the_limit_fail_in_a_row() {
        let mut looks = Looks::new();

        for _ in 1..MAX_FAILED_LOOKS {
            looks.record(failed_look()).unwrap();
        }

        assert_eq!(looks.state(), ProcessState::Running);
    }

    #[test]
    fn should_pass_the_failure_on_when_the_limit_of_looks_fail_in_a_row() {
        let mut looks = Looks::new();
        for _ in 1..MAX_FAILED_LOOKS {
            looks.record(failed_look()).unwrap();
        }

        let last = looks.record(failed_look());

        assert!(last.is_err());
        assert_eq!(looks.state(), ProcessState::Running);
    }

    #[test]
    fn should_count_the_failed_looks_anew_when_a_look_succeeds_in_between() {
        let mut looks = Looks::new();
        for _ in 1..MAX_FAILED_LOOKS {
            looks.record(failed_look()).unwrap();
        }
        looks.record(Ok(Liveness::Running)).unwrap();

        for _ in 1..MAX_FAILED_LOOKS {
            looks.record(failed_look()).unwrap();
        }

        assert_eq!(looks.state(), ProcessState::Running);
    }

    #[rstest::rstest]
    #[case::zombie(Liveness::Zombie)]
    #[case::gone(Liveness::Gone)]
    fn should_see_the_exit_and_look_no_more_when_the_process_is_not_running(
        #[case] seen: Liveness,
    ) {
        let mut looks = Looks::new();

        looks.record(Ok(seen)).unwrap();

        assert_eq!(looks.state(), ProcessState::Exited(None));
        assert!(!looks.is_due());
    }

    const OWN_PID: u32 = 100;
    const OWN_GROUP: u32 = 90;

    #[rstest::rstest]
    #[case::pid_zero(0, KillScope::Process)]
    #[case::group_zero(0, KillScope::Tree)]
    #[case::beyond_pid_t(u32::MAX, KillScope::Process)]
    #[case::first_beyond_pid_t(1 << 31, KillScope::Tree)]
    #[case::the_launcher(OWN_PID, KillScope::Process)]
    #[case::the_launcher_tree(OWN_PID, KillScope::Tree)]
    #[case::the_launcher_group(OWN_GROUP, KillScope::Tree)]
    fn should_refuse_to_signal_when_the_signal_would_reach_the_launcher(
        #[case] pid: u32,
        #[case] kill_scope: KillScope,
    ) {
        let target = kill_target(pid, kill_scope, OWN_PID, OWN_GROUP);

        assert_eq!(target.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[rstest::rstest]
    #[case::a_process(4242, KillScope::Process)]
    #[case::a_group(4242, KillScope::Tree)]
    fn should_signal_the_pid_when_it_is_a_game_apart_from_the_launcher(
        #[case] pid: u32,
        #[case] kill_scope: KillScope,
    ) {
        let target = kill_target(pid, kill_scope, OWN_PID, OWN_GROUP);

        assert_eq!(target.unwrap(), pid.cast_signed());
    }

    #[tokio::test]
    async fn should_read_the_same_identity_twice_when_the_process_is_still_running() {
        let probe = OsProcessProbe::new();
        let own = probe
            .own_identity()
            .expect("the launcher's own identity")
            .clone();

        let again = probe.identity_of(std::process::id()).await.unwrap();

        assert!(own.matches(&again), "{own:?} != {again:?}");
        assert_ne!(own.start_stamp(), 0);
    }

    #[tokio::test]
    async fn should_read_a_later_start_of_the_same_boot_when_the_launcher_starts_a_child() {
        let probe = OsProcessProbe::new();
        let mut child = long_running();

        let identity = probe.identity_of(child.id()).await;
        let _ = child.kill();
        let _ = child.wait();

        let identity = identity.unwrap();
        let own = probe.own_identity().unwrap();
        assert_eq!(identity.pid(), child.id());
        assert!(
            identity.start_stamp() >= own.start_stamp(),
            "{identity:?} before {own:?}"
        );
        assert_eq!(identity.boot_id(), own.boot_id());
        #[cfg(target_os = "linux")]
        assert!(!own.boot_id().is_empty());
    }

    /// A program that runs long and has nothing to do with the test binary.
    fn long_running() -> Child {
        #[cfg(windows)]
        let mut command = Command::new("ping");
        #[cfg(windows)]
        command.args(["-n", "120", "127.0.0.1"]);
        #[cfg(not(windows))]
        let mut command = Command::new("sleep");
        #[cfg(not(windows))]
        command.arg("120");
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .unwrap()
    }

    fn record_of(process: OsProcessIdentity, kill_scope: KillScope) -> ProcessRecord {
        ProcessRecord {
            id: Uuid::new_v4(),
            instance_id: InstanceId::generate(),
            owner: process.clone(),
            process,
            started_at: Utc::now(),
            kill_scope,
            post_exit_command: String::new(),
            playtime_credited_until: Utc::now(),
        }
    }

    async fn adopt_child(
        probe: &OsProcessProbe,
        child: &Child,
        kill_scope: KillScope,
    ) -> Box<dyn AdoptedProcess> {
        let identity = probe.identity_of(child.id()).await.unwrap();
        probe
            .adopt(&record_of(identity, kill_scope))
            .await
            .expect("the OS tells about the child")
            .expect("the running child is adopted")
    }

    /// Poll until `done` holds. A condition polled, not a delay guessed: how long a process
    /// takes to start or to die is up to the OS, so only the give-up deadline is generous.
    async fn eventually(what: &str, mut done: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while !done() {
            assert!(Instant::now() < deadline, "timed out waiting until {what}");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn wait_for_exit(process: &mut dyn AdoptedProcess) -> ProcessState {
        let mut state = ProcessState::Running;
        eventually("the adopted process has exited", || {
            state = process.try_wait().unwrap();
            state != ProcessState::Running
        })
        .await;
        state
    }

    #[tokio::test]
    async fn should_follow_a_running_process_when_its_pid_and_start_match() {
        let probe = OsProcessProbe::new();
        let mut child = long_running();

        let mut adopted = adopt_child(&probe, &child, KillScope::Process).await;
        let state = adopted.try_wait();
        let record = record_of(
            probe.identity_of(child.id()).await.unwrap(),
            KillScope::Process,
        );
        let alive = probe.is_alive(&record).await;
        let _ = child.kill();
        let _ = child.wait();

        assert_eq!(state.unwrap(), ProcessState::Running);
        assert!(alive.unwrap());
    }

    #[tokio::test]
    async fn should_adopt_nothing_when_the_pid_belongs_to_a_process_of_another_start() {
        let probe = OsProcessProbe::new();
        let mut child = long_running();
        let identity = probe.identity_of(child.id()).await.unwrap();
        let other = OsProcessIdentity::new(
            identity.pid(),
            identity.start_stamp() - 1,
            identity.boot_id().to_owned(),
        );
        let record = record_of(other, KillScope::Process);

        let adopted = probe.adopt(&record).await;
        let alive = probe.is_alive(&record).await;
        let _ = child.kill();
        let _ = child.wait();

        assert!(adopted.unwrap().is_none());
        assert!(!alive.unwrap());
    }

    #[tokio::test]
    async fn should_adopt_nothing_when_the_process_is_gone() {
        let probe = OsProcessProbe::new();
        let mut child = long_running();
        let identity = probe.identity_of(child.id()).await.unwrap();
        let _ = child.kill();
        let _ = child.wait();
        // On Windows `child` still holds a handle, which keeps the ended process around.
        let record = record_of(identity, KillScope::Process);

        assert!(probe.adopt(&record).await.unwrap().is_none());
        assert!(!probe.is_alive(&record).await.unwrap());
    }

    #[tokio::test]
    async fn should_stop_the_adopted_process_when_it_is_killed() {
        let probe = OsProcessProbe::new();
        let mut child = long_running();
        let mut adopted = adopt_child(&probe, &child, KillScope::Process).await;

        adopted.start_kill().unwrap();

        let status = child.wait().unwrap();
        assert!(!status.success(), "{status:?}");
        let state = wait_for_exit(adopted.as_mut()).await;
        #[cfg(windows)]
        assert_eq!(
            state,
            ProcessState::Exited(Some(std::os::windows::process::ExitStatusExt::from_raw(1)))
        );
        #[cfg(unix)]
        assert_eq!(state, ProcessState::Exited(None));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn should_read_the_exit_code_when_the_adopted_process_exits() {
        use std::os::windows::process::{CommandExt, ExitStatusExt};
        use std::process::ExitStatus;

        let probe = OsProcessProbe::new();
        let mut child = Command::new("cmd")
            .raw_arg("/c ping -n 3 127.0.0.1 >nul & exit 3")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let mut adopted = adopt_child(&probe, &child, KillScope::Process).await;

        let state = wait_for_exit(adopted.as_mut()).await;
        let _ = child.wait();

        assert_eq!(state, ProcessState::Exited(Some(ExitStatus::from_raw(3))));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn should_see_the_exit_when_the_adopted_process_is_reaped() {
        let probe = OsProcessProbe::new();
        let mut child = long_running();
        let mut adopted = adopt_child(&probe, &child, KillScope::Process).await;

        let _ = child.kill();
        let _ = child.wait();

        assert_eq!(
            wait_for_exit(adopted.as_mut()).await,
            ProcessState::Exited(None)
        );
    }

    /// A process whose parent has not reaped it yet has ended all the same.
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn should_see_the_exit_when_the_adopted_process_is_a_zombie() {
        let probe = OsProcessProbe::new();
        let mut child = long_running();
        let mut adopted = adopt_child(&probe, &child, KillScope::Process).await;
        let pid = child.id();

        let _ = child.kill();
        eventually("the child is a zombie", || {
            matches!(read_proc_stat(pid), Ok(Some((_, 'Z'))))
        })
        .await;
        let state = wait_for_exit(adopted.as_mut()).await;
        let _ = child.wait();

        assert_eq!(state, ProcessState::Exited(None));
    }

    /// A shell that starts long-running grandchildren of the test and waits on them.
    const SCRIPT_WITH_GRANDCHILDREN: &str = if cfg!(windows) {
        "start /b ping -n 600 127.0.0.1 > nul & ping -n 600 127.0.0.1 > nul"
    } else {
        "sleep 600 & sleep 600 & wait"
    };

    /// The grandchild program [`SCRIPT_WITH_GRANDCHILDREN`] starts.
    const GRANDCHILD_NAME: &str = if cfg!(windows) { "ping" } else { "sleep" };

    /// A live process, by pid and start time, so a reused pid is not mistaken for it.
    type ProcessKey = (sysinfo::Pid, u64);

    fn is_live(process: &sysinfo::Process) -> bool {
        process.status() != sysinfo::ProcessStatus::Zombie
    }

    /// The live processes named [`GRANDCHILD_NAME`] right below `parent`.
    fn grandchildren(system: &mut sysinfo::System, parent: u32) -> Vec<ProcessKey> {
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let parent = sysinfo::Pid::from_u32(parent);
        system
            .processes()
            .iter()
            .filter(|(_, process)| process.parent() == Some(parent) && is_live(process))
            // Windows names it `PING.EXE`.
            .filter(|(_, process)| {
                process
                    .name()
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .starts_with(GRANDCHILD_NAME)
            })
            .map(|(pid, process)| (*pid, process.start_time()))
            .collect()
    }

    fn any_live(system: &mut sysinfo::System, processes: &[ProcessKey]) -> bool {
        let pids: Vec<_> = processes.iter().map(|(pid, _)| *pid).collect();
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&pids), true);
        processes.iter().any(|(pid, started)| {
            system
                .process(*pid)
                .is_some_and(|process| process.start_time() == *started && is_live(process))
        })
    }

    /// The tree is started without the job object or the process group of the launcher that
    /// started it: that launcher is gone, and only the group survives it on Unix.
    #[tokio::test]
    async fn should_stop_the_grandchildren_when_the_adopted_tree_is_killed() {
        let probe = OsProcessProbe::new();
        #[cfg(windows)]
        let mut command = {
            use std::os::windows::process::CommandExt;
            let mut command = Command::new("cmd");
            command.raw_arg(format!("/c {SCRIPT_WITH_GRANDCHILDREN}"));
            command
        };
        #[cfg(unix)]
        let mut command = {
            use std::os::unix::process::CommandExt;
            let mut command = Command::new("sh");
            command
                .args(["-c", SCRIPT_WITH_GRANDCHILDREN])
                .process_group(0);
            command
        };
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let mut system = sysinfo::System::new().unwrap();
        let mut running = Vec::new();
        eventually("the grandchildren are running", || {
            running = grandchildren(&mut system, child.id());
            running.len() == 2
        })
        .await;
        let mut adopted = adopt_child(&probe, &child, KillScope::Tree).await;

        adopted.start_kill().unwrap();

        // Looked at before the shell is waited for: it waits on a grandchild that was missed.
        let deadline = Instant::now() + Duration::from_secs(30);
        while any_live(&mut system, &running) && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let survivors: Vec<_> = running
            .iter()
            .filter(|key| any_live(&mut system, std::slice::from_ref(key)))
            .collect();
        for (pid, _) in &survivors {
            if let Some(process) = system.process(*pid) {
                process.kill();
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        assert!(survivors.is_empty(), "still running: {survivors:?}");
    }
}
