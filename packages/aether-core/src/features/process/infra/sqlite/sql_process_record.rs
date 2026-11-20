use chrono::{DateTime, SecondsFormat, Utc};
use tracing::warn;
use uuid::Uuid;

use crate::features::instance::InstanceId;
use crate::features::process::{KillScope, OsProcessIdentity, ProcessRecord};

/// A row of `game_processes`. The start stamps are `u64` values kept bit for bit in the signed
/// column, so that a stored stamp compares exactly with a fresh one.
pub struct SqlProcessRecord {
    pub id: String,
    pub instance_id: String,
    pub pid: i64,
    pub start_stamp: i64,
    pub boot_id: String,
    pub started_at: String,
    pub kill_scope: String,
    pub post_exit_command: String,
    pub playtime_credited_until: String,
    pub owner_pid: i64,
    pub owner_start_stamp: i64,
    pub owner_boot_id: String,
}

pub fn format_timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

impl From<&ProcessRecord> for SqlProcessRecord {
    fn from(record: &ProcessRecord) -> Self {
        Self {
            id: record.id.to_string(),
            instance_id: record.instance_id.as_str().to_owned(),
            pid: i64::from(record.process.pid()),
            start_stamp: record.process.start_stamp().cast_signed(),
            boot_id: record.process.boot_id().to_owned(),
            started_at: format_timestamp(record.started_at),
            kill_scope: match record.kill_scope {
                KillScope::Process => "process",
                KillScope::Tree => "tree",
            }
            .to_owned(),
            post_exit_command: record.post_exit_command.clone(),
            playtime_credited_until: format_timestamp(record.playtime_credited_until),
            owner_pid: i64::from(record.owner.pid()),
            owner_start_stamp: record.owner.start_stamp().cast_signed(),
            owner_boot_id: record.owner.boot_id().to_owned(),
        }
    }
}

impl SqlProcessRecord {
    /// The record, or `None` with a warning when a column does not hold what the launcher
    /// writes there: such a row stands for no game that can be found again.
    pub fn into_record(self) -> Option<ProcessRecord> {
        let id = &self.id;
        Some(ProcessRecord {
            id: parse_stored_id(id)?,
            instance_id: parse_stored_instance_id(id, self.instance_id)?,
            process: OsProcessIdentity::new(
                parse_stored_pid(id, "pid", self.pid)?,
                self.start_stamp.cast_unsigned(),
                self.boot_id,
            ),
            started_at: parse_stored_timestamp(id, "started_at", &self.started_at)?,
            kill_scope: parse_stored_kill_scope(id, &self.kill_scope)?,
            post_exit_command: self.post_exit_command,
            playtime_credited_until: parse_stored_timestamp(
                id,
                "playtime_credited_until",
                &self.playtime_credited_until,
            )?,
            owner: OsProcessIdentity::new(
                parse_stored_pid(id, "owner_pid", self.owner_pid)?,
                self.owner_start_stamp.cast_unsigned(),
                self.owner_boot_id,
            ),
        })
    }
}

fn parse_stored_id(stored: &str) -> Option<Uuid> {
    Uuid::parse_str(stored)
        .inspect_err(|err| {
            warn!(record = stored, error = %err, "Skipping a game record with an invalid id");
        })
        .ok()
}

fn parse_stored_instance_id(record: &str, stored: String) -> Option<InstanceId> {
    InstanceId::try_from(stored)
        .inspect_err(|err| {
            warn!(record, error = %err, "Skipping a game record with an invalid instance id");
        })
        .ok()
}

fn parse_stored_pid(record: &str, column: &str, stored: i64) -> Option<u32> {
    u32::try_from(stored)
        .inspect_err(|_| {
            warn!(
                record,
                column, stored, "Skipping a game record with an invalid pid"
            );
        })
        .ok()
}

fn parse_stored_timestamp(record: &str, column: &str, stored: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(stored)
        .map(|parsed| parsed.with_timezone(&Utc))
        .inspect_err(|err| {
            warn!(record, column, stored, error = %err, "Skipping a game record with an invalid time");
        })
        .ok()
}

fn parse_stored_kill_scope(record: &str, stored: &str) -> Option<KillScope> {
    match stored {
        "process" => Some(KillScope::Process),
        "tree" => Some(KillScope::Tree),
        _ => {
            warn!(
                record,
                stored, "Skipping a game record with an unknown kill scope"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> ProcessRecord {
        let now = Utc::now();
        ProcessRecord {
            id: Uuid::new_v4(),
            instance_id: InstanceId::try_from("a").unwrap(),
            process: OsProcessIdentity::new(42, u64::MAX, "boot".to_owned()),
            started_at: now,
            kill_scope: KillScope::Tree,
            post_exit_command: String::new(),
            playtime_credited_until: now,
            owner: OsProcessIdentity::new(7, 1, String::new()),
        }
    }

    #[test]
    fn should_keep_a_start_stamp_that_does_not_fit_the_signed_column() {
        let record = record();

        let read = SqlProcessRecord::from(&record).into_record().unwrap();

        assert_eq!(read.process.start_stamp(), u64::MAX);
    }

    #[test]
    fn should_skip_the_record_when_its_kill_scope_is_unknown() {
        let mut row = SqlProcessRecord::from(&record());
        row.kill_scope = "group".to_owned();

        assert!(row.into_record().is_none());
    }

    #[test]
    fn should_skip_the_record_when_its_pid_is_negative() {
        let mut row = SqlProcessRecord::from(&record());
        row.pid = -1;

        assert!(row.into_record().is_none());
    }
}
