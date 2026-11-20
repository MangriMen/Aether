//! A game the launcher did not start, followed through a handle to its process. While the
//! handle is open the process object stays, so the pid cannot pass to another process and the
//! exit code can be read even though the game is not a child of this launcher.

use std::collections::HashSet;
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::os::windows::process::ExitStatusExt;
use std::process::ExitStatus;

use tracing::{debug, warn};
use windows::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, FILETIME, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_ACCESS_RIGHTS,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE, TerminateProcess,
    WaitForSingleObject,
};

use crate::features::process::{AdoptedProcess, KillScope, OsProcessIdentity, ProcessState};

/// The exit code of a stopped game, the one `TerminateProcess` gives a started game too.
const KILLED_EXIT_CODE: u32 = 1;

/// Enough to follow a process: read its times and exit code, and wait for it.
const WATCH: PROCESS_ACCESS_RIGHTS =
    PROCESS_ACCESS_RIGHTS(PROCESS_QUERY_LIMITED_INFORMATION.0 | PROCESS_SYNCHRONIZE.0);

#[derive(Debug)]
pub(super) struct WindowsAdoptedProcess {
    handle: OwnedHandle,
    pid: u32,
    /// The creation time of the process, in 100 ns intervals since 1601.
    created: u64,
    kill_scope: KillScope,
    /// False for a game that runs elevated: a launcher that does not may follow it, not stop it.
    can_terminate: bool,
}

impl WindowsAdoptedProcess {
    /// Opens the process of `identity`; `None` when it has ended or the pid is not in use or
    /// belongs to a process of another start.
    pub(super) fn open(identity: &OsProcessIdentity, kill_scope: KillScope) -> Option<Self> {
        let pid = identity.pid();
        let (handle, can_terminate) = match open_process(pid, WATCH | PROCESS_TERMINATE) {
            Ok(handle) => (handle, true),
            // A process holds the pid, it is not gone: an elevated game refuses to be stopped by
            // a launcher that is not, yet lets it watch.
            Err(err) if err.code() == ERROR_ACCESS_DENIED.to_hresult() => {
                match open_process(pid, WATCH) {
                    Ok(handle) => (handle, false),
                    Err(err) => {
                        warn!(pid, error = %err, "Could not open the game process to follow it");
                        return None;
                    }
                }
            }
            Err(err) => {
                // The pid is not in use: the game is gone.
                if err.code() != ERROR_INVALID_PARAMETER.to_hresult() {
                    warn!(pid, error = %err, "Could not open the game process to follow it");
                }
                return None;
            }
        };
        let created = creation_time(&handle)
            .inspect_err(|err| {
                warn!(pid, error = %err, "Could not read when the game process started");
            })
            .ok()?;
        // Compared on the handle that is kept, so the pid cannot change hands in between.
        let current = OsProcessIdentity::new(pid, start_stamp(created), String::new());
        if !identity.matches(&current) {
            return None;
        }
        // An ended process is kept as long as anyone holds a handle to it, yet it is gone.
        let exited = has_exited(&handle)
            .inspect_err(|err| {
                warn!(pid, error = %err, "Could not tell whether the game process still runs");
            })
            .ok()?;
        (!exited).then_some(Self {
            handle,
            pid,
            created,
            kill_scope,
            can_terminate,
        })
    }
}

impl AdoptedProcess for WindowsAdoptedProcess {
    fn try_wait(&mut self) -> io::Result<ProcessState> {
        if !has_exited(&self.handle)? {
            return Ok(ProcessState::Running);
        }
        // Read only once the process is signalled: before that the code is STILL_ACTIVE, which
        // a game may also exit with.
        let mut code = 0;
        // SAFETY: the handle is open with PROCESS_QUERY_LIMITED_INFORMATION, and `code` lives
        // through the call.
        unsafe { GetExitCodeProcess(raw(&self.handle), &raw mut code) }?;
        Ok(ProcessState::Exited(Some(ExitStatus::from_raw(code))))
    }

    fn start_kill(&mut self) -> io::Result<()> {
        if !self.can_terminate {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "cannot stop an elevated game",
            ));
        }
        // Found before the root goes: the tree is followed by parent pids.
        let descendants = match self.kill_scope {
            KillScope::Process => Vec::new(),
            KillScope::Tree => descendants(self.pid, self.created),
        };
        terminate(&self.handle)?;
        for descendant in &descendants {
            if let Err(err) = terminate(descendant) {
                debug!(pid = self.pid, error = %err, "Could not stop a process of the game tree");
            }
        }
        Ok(())
    }
}

fn raw(handle: &OwnedHandle) -> HANDLE {
    HANDLE(handle.as_raw_handle())
}

fn open_process(pid: u32, access: PROCESS_ACCESS_RIGHTS) -> windows::core::Result<OwnedHandle> {
    // SAFETY: `OpenProcess` has no preconditions. The handle it returns belongs to nobody else,
    // so `OwnedHandle` is its only owner and closes it once.
    unsafe { OpenProcess(access, false, pid).map(|handle| OwnedHandle::from_raw_handle(handle.0)) }
}

/// The creation time of the process, in 100 ns intervals since 1601.
fn creation_time(handle: &OwnedHandle) -> io::Result<u64> {
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    // SAFETY: the handle is open with PROCESS_QUERY_LIMITED_INFORMATION, and every pointer is to
    // a FILETIME that lives through the call.
    unsafe {
        GetProcessTimes(
            raw(handle),
            &raw mut creation,
            &raw mut exit,
            &raw mut kernel,
            &raw mut user,
        )
    }?;
    Ok((u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime))
}

/// The start stamp of a process created at `created`, as `sysinfo` reports it and the record
/// of the game keeps it: whole seconds since the Unix epoch.
fn start_stamp(created: u64) -> u64 {
    const SECONDS_FROM_1601_TO_1970: u64 = 11_644_473_600;
    (created / 10_000_000).saturating_sub(SECONDS_FROM_1601_TO_1970)
}

fn has_exited(handle: &OwnedHandle) -> io::Result<bool> {
    // SAFETY: the handle is open with PROCESS_SYNCHRONIZE; a zero timeout only polls.
    let waited = unsafe { WaitForSingleObject(raw(handle), 0) };
    match waited {
        WAIT_OBJECT_0 => Ok(true),
        WAIT_TIMEOUT => Ok(false),
        _ => Err(io::Error::last_os_error()),
    }
}

/// Stops the process. One that has already exited cannot be terminated, and is not an error.
fn terminate(handle: &OwnedHandle) -> io::Result<()> {
    // SAFETY: the handle is open with PROCESS_TERMINATE.
    let terminated = unsafe { TerminateProcess(raw(handle), KILLED_EXIT_CODE) };
    match terminated {
        Ok(()) => Ok(()),
        Err(_) if has_exited(handle)? => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// The processes below `root`, opened to be stopped. Best effort: the job object that held the
/// tree went with the launcher that made it, so the tree is found by parent pids. A process
/// whose parent has already ended is not found, and one created before its supposed parent is a
/// stranger that holds a reused pid.
fn descendants(root: u32, root_created: u64) -> Vec<OwnedHandle> {
    let parents = match parent_pids() {
        Ok(parents) => parents,
        Err(err) => {
            warn!(pid = root, error = %err, "Could not list the processes of the game tree");
            return Vec::new();
        }
    };
    let mut seen = HashSet::from([root]);
    let mut to_visit = vec![(root, root_created)];
    let mut found = Vec::new();
    while let Some((parent, parent_created)) = to_visit.pop() {
        for &(pid, _) in parents.iter().filter(|(_, ppid)| *ppid == parent) {
            if !seen.insert(pid) {
                continue;
            }
            let Ok(handle) = open_process(pid, WATCH | PROCESS_TERMINATE) else {
                continue;
            };
            match creation_time(&handle) {
                Ok(created) if created >= parent_created => {
                    to_visit.push((pid, created));
                    found.push(handle);
                }
                _ => {}
            }
        }
    }
    found
}

/// Every running process as `(pid, parent pid)`.
fn parent_pids() -> io::Result<Vec<(u32, u32)>> {
    // SAFETY: `CreateToolhelp32Snapshot` has no preconditions. The snapshot handle belongs to
    // nobody else, so `OwnedHandle` is its only owner and closes it once.
    let snapshot = unsafe {
        CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map(|handle| OwnedHandle::from_raw_handle(handle.0))
    }?;
    let mut entry = PROCESSENTRY32W {
        dwSize: u32::try_from(size_of::<PROCESSENTRY32W>()).map_err(io::Error::other)?,
        ..PROCESSENTRY32W::default()
    };
    let mut parents = Vec::new();
    // SAFETY: the snapshot is open, and `entry` lives through the call with its size set, as
    // the walk requires.
    let mut next = unsafe { Process32FirstW(raw(&snapshot), &raw mut entry) };
    while next.is_ok() {
        parents.push((entry.th32ProcessID, entry.th32ParentProcessID));
        // SAFETY: as above.
        next = unsafe { Process32NextW(raw(&snapshot), &raw mut entry) };
    }
    Ok(parents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_count_whole_seconds_from_the_unix_epoch_when_the_start_is_turned_into_a_stamp() {
        // 1970-01-01T00:00:01.9 as a FILETIME.
        let created = (11_644_473_600 + 1) * 10_000_000 + 9_000_000;

        assert_eq!(start_stamp(created), 1);
    }
}
