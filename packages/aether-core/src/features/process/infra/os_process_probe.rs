use std::io;

use async_trait::async_trait;
use tracing::warn;

use crate::features::process::{OsProcessIdentity, ProcessProbe};

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
}

#[cfg(target_os = "linux")]
fn read_identity(pid: u32) -> io::Result<OsProcessIdentity> {
    let path = format!("/proc/{pid}/stat");
    let stat = std::fs::read_to_string(&path)?;
    let start_ticks = parse_start_ticks(&stat).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("no start time in {path}"),
        )
    })?;
    let boot_id = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?;
    Ok(OsProcessIdentity::new(
        pid,
        start_ticks,
        boot_id.trim().to_owned(),
    ))
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

#[cfg(not(target_os = "linux"))]
fn read_identity(pid: u32) -> io::Result<OsProcessIdentity> {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

    let mut system = System::new().map_err(io::Error::other)?;
    let target = Pid::from_u32(pid);
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[target]),
        false,
        ProcessRefreshKind::nothing(),
    );
    // `sysinfo` reports 0 for a process it could not open.
    let start = system
        .process(target)
        .map(sysinfo::Process::start_time)
        .filter(|&start| start != 0)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("no start time for process {pid}"),
            )
        })?;
    Ok(OsProcessIdentity::new(pid, start, String::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_the_start_ticks_when_the_process_name_holds_spaces_and_parentheses() {
        let stat = "4242 (java (main) x) S 1 4242 4242 0 -1 4194560 100 0 0 0 5 2 0 0 20 0 \
                    30 0 987654 123456789 1000 18446744073709551615";

        assert_eq!(parse_start_ticks(stat), Some(987_654));
    }

    #[test]
    fn should_read_no_start_ticks_when_the_line_is_cut_short() {
        assert_eq!(parse_start_ticks("4242 (java) S 1 4242"), None);
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
    async fn should_tell_the_child_from_the_launcher_when_both_run() {
        let probe = OsProcessProbe::new();
        #[cfg(windows)]
        let mut command = std::process::Command::new("ping");
        #[cfg(windows)]
        command.args(["-n", "30", "127.0.0.1"]);
        #[cfg(not(windows))]
        let mut command = std::process::Command::new("sleep");
        #[cfg(not(windows))]
        command.arg("30");
        let mut child = command.stdout(std::process::Stdio::null()).spawn().unwrap();

        let identity = probe.identity_of(child.id()).await;
        let _ = child.kill();
        let _ = child.wait();

        let identity = identity.unwrap();
        assert_eq!(identity.pid(), child.id());
        assert!(!identity.matches(probe.own_identity().unwrap()));
    }
}
