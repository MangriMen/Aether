use chrono::{DateTime, Duration, Utc};
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

/// The most playtime credited for a run that ended while no launcher followed it. Its end is
/// guessed from the last write to its log, which may be off by much: a wrong guess must cost
/// little.
pub const MAX_UNFOLLOWED_END_PLAYTIME: Duration = Duration::hours(24);

/// The playtime of a run that ended while no launcher followed it: from `credited_until` to the
/// last write the game made to its log, not past `now`, and at most
/// [`MAX_UNFOLLOWED_END_PLAYTIME`]. Nothing when no such write is known: time is rather left
/// uncredited than credited for a game that was not running.
pub fn unfollowed_end_playtime(
    credited_until: DateTime<Utc>,
    last_log_write: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Duration {
    let Some(last_log_write) = last_log_write else {
        return Duration::zero();
    };
    let end = last_log_write.min(now);
    let played = end.signed_duration_since(credited_until);
    played.clamp(Duration::zero(), MAX_UNFOLLOWED_END_PLAYTIME)
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

    fn at(seconds: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000 + seconds, 0).unwrap()
    }

    #[test]
    fn should_credit_up_to_the_last_log_write_when_the_game_ended_unfollowed() {
        let played = unfollowed_end_playtime(at(0), Some(at(600)), at(3600));

        assert_eq!(played, Duration::seconds(600));
    }

    #[test]
    fn should_credit_up_to_now_when_the_log_was_written_later_than_now() {
        let played = unfollowed_end_playtime(at(0), Some(at(7200)), at(3600));

        assert_eq!(played, Duration::seconds(3600));
    }

    #[test]
    fn should_credit_nothing_when_no_log_write_is_known() {
        assert_eq!(
            unfollowed_end_playtime(at(0), None, at(3600)),
            Duration::zero()
        );
    }

    #[test]
    fn should_credit_nothing_when_the_log_was_last_written_before_the_mark() {
        assert_eq!(
            unfollowed_end_playtime(at(600), Some(at(300)), at(3600)),
            Duration::zero()
        );
    }

    #[test]
    fn should_credit_at_most_a_day_when_the_log_was_written_much_later() {
        let week = 7 * 24 * 3600;

        let played = unfollowed_end_playtime(at(0), Some(at(week)), at(week));

        assert_eq!(played, MAX_UNFOLLOWED_END_PLAYTIME);
    }
}
