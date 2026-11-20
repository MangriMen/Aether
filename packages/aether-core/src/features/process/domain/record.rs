use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::KillScope;
use crate::features::instance::InstanceId;

/// A running process as the OS tells it apart from a later one that reuses its pid.
///
/// The start stamp is what the OS reports for the start of the process: clock ticks since boot
/// on Linux, where the boot id keeps a stamp of one boot from matching one of another; the
/// creation time in seconds elsewhere, with an empty boot id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsProcessIdentity {
    pid: u32,
    start_stamp: u64,
    boot_id: String,
}

impl OsProcessIdentity {
    pub fn new(pid: u32, start_stamp: u64, boot_id: String) -> Self {
        Self {
            pid,
            start_stamp,
            boot_id,
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn start_stamp(&self) -> u64 {
        self.start_stamp
    }

    pub fn boot_id(&self) -> &str {
        &self.boot_id
    }

    /// Whether `other` is the same process. The comparison is exact: the stamps are the raw
    /// values of the OS, so a tolerance would only let a process that reused the pid through.
    pub fn matches(&self, other: &Self) -> bool {
        self == other
    }
}

/// A started game as the launcher keeps it, so that a later launcher can find it running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRecord {
    /// The id of the run, [`MinecraftProcessMetadata::uuid`](super::MinecraftProcessMetadata::uuid).
    pub id: Uuid,
    pub instance_id: InstanceId,
    pub process: OsProcessIdentity,
    pub started_at: DateTime<Utc>,
    pub kill_scope: KillScope,
    pub post_exit_command: String,
    /// The moment up to which the playtime of the run is added to the instance.
    pub playtime_credited_until: DateTime<Utc>,
    /// The launcher that follows the game.
    pub owner: OsProcessIdentity,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(pid: u32, start_stamp: u64, boot_id: &str) -> OsProcessIdentity {
        OsProcessIdentity::new(pid, start_stamp, boot_id.to_owned())
    }

    #[test]
    fn should_match_when_pid_start_and_boot_are_the_same() {
        assert!(identity(42, 1000, "boot").matches(&identity(42, 1000, "boot")));
    }

    #[test]
    fn should_not_match_when_the_pid_differs() {
        assert!(!identity(42, 1000, "boot").matches(&identity(43, 1000, "boot")));
    }

    #[test]
    fn should_not_match_when_the_start_differs_by_one_tick() {
        assert!(!identity(42, 1000, "boot").matches(&identity(42, 1001, "boot")));
    }

    #[test]
    fn should_not_match_when_the_boot_differs() {
        assert!(!identity(42, 1000, "boot").matches(&identity(42, 1000, "other")));
    }
}
