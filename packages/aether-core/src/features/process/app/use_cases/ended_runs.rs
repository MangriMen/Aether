use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use crate::core::domain::error_chain;
use crate::features::{
    game_log::{ListGameLogSourcesUseCasePort, LogSourceKind},
    instance::{InstanceId, InstanceStorage, InstanceStorageExt},
    process::{
        OsProcessIdentity, ProcessProbe, ProcessRecord, ProcessRecordStorage, ProcessStorage,
        domain::unfollowed_end_playtime,
    },
};
use crate::shared::error::DatabaseError;

use super::SkippedHookNotices;

/// The log the game itself writes up to its end, unlike the console log the launcher wrote.
const GAME_LOG: &str = "latest.log";

/// Forgets the record of a game that ended while no launcher followed it, and credits what can
/// be told of the end of its run.
pub(super) struct EndedRuns {
    pub(super) records: Arc<dyn ProcessRecordStorage>,
    pub(super) process_storage: Arc<dyn ProcessStorage>,
    pub(super) instance_storage: Arc<dyn InstanceStorage>,
    pub(super) game_logs: Arc<dyn ListGameLogSourcesUseCasePort>,
    pub(super) probe: Arc<dyn ProcessProbe>,
    pub(super) notices: Arc<SkippedHookNotices>,
}

impl EndedRuns {
    /// Removes the record of the ended game. Only the launcher that removed it credits the rest
    /// of the run and tells of the skipped post-exit hook: two launchers that find the same
    /// record do it once.
    pub(super) async fn forget(&self, record: &ProcessRecord) -> Result<(), DatabaseError> {
        let now = Utc::now();
        if !self.records.remove(record.id).await? {
            return Ok(());
        }
        let playtime = self.unfollowed_playtime(record, now).await;
        tracing::info!(
            process_id = %record.id,
            instance_id = %record.instance_id,
            credited_seconds = playtime.num_seconds(),
            "The game ended while no launcher followed it"
        );
        self.credit(&record.instance_id, playtime).await;
        // A hook run hours late would surprise more than one that was not run.
        if !record.post_exit_command.trim().is_empty() {
            tracing::info!(
                process_id = %record.id,
                instance_id = %record.instance_id,
                "The post-exit hook of the game was not run: it ended while no launcher followed it"
            );
            self.notices.notify(&record.instance_id).await;
        }
        Ok(())
    }

    /// Adds `playtime` to the instance; a failure leaves it uncredited.
    pub(super) async fn credit(&self, instance_id: &InstanceId, playtime: Duration) {
        let seconds = playtime.num_seconds();
        if seconds <= 0 {
            return;
        }
        let result = self
            .instance_storage
            .upsert_with(instance_id, |instance| {
                instance.time_played += seconds.cast_unsigned();
                Ok(())
            })
            .await;
        if let Err(err) = result {
            tracing::warn!(
                %instance_id,
                error = %error_chain(&err),
                "Could not credit the playtime"
            );
        }
    }

    /// The rest of the run up to the last write to its log. Nothing when that write may be of
    /// another run: a later one of the same instance writes the same log, and a run of an
    /// earlier boot is not told apart from the boot it ended in.
    async fn unfollowed_playtime(&self, record: &ProcessRecord, now: DateTime<Utc>) -> Duration {
        if is_of_another_boot(&record.process, self.probe.own_identity())
            || self.later_run_may_exist(record).await
        {
            return Duration::zero();
        }
        let last_log_write = self.last_log_write(&record.instance_id).await;
        unfollowed_end_playtime(record.playtime_credited_until, last_log_write, now)
    }

    async fn later_run_may_exist(&self, record: &ProcessRecord) -> bool {
        let later_record = match self.records.list_for_instance(&record.instance_id).await {
            Ok(records) => records
                .iter()
                .any(|other| other.id != record.id && other.started_at > record.started_at),
            Err(_) => true,
        };
        let later_followed = match self.process_storage.list_metadata().await {
            Ok(processes) => processes.iter().any(|process| {
                process.uuid() != record.id
                    && *process.instance_id() == record.instance_id
                    && *process.start_time() > record.started_at
            }),
            Err(_) => true,
        };
        later_record || later_followed
    }

    async fn last_log_write(&self, instance_id: &InstanceId) -> Option<DateTime<Utc>> {
        let sources = self
            .game_logs
            .execute(instance_id.clone())
            .await
            .inspect_err(|err| {
                tracing::debug!(
                    %instance_id,
                    error = %error_chain(err),
                    "Could not list the logs of the instance"
                );
            })
            .ok()?;
        let modified = sources
            .into_iter()
            .find(|log| log.source.kind() == LogSourceKind::Game && log.source.name() == GAME_LOG)?
            .modified?;
        DateTime::from_timestamp_millis(modified)
    }
}

/// Whether `process` may be of another boot than the launcher's. Only Linux tells boots apart;
/// when the launcher does not know its own boot, it cannot tell.
fn is_of_another_boot(process: &OsProcessIdentity, own: Option<&OsProcessIdentity>) -> bool {
    if process.boot_id().is_empty() {
        return false;
    }
    own.is_none_or(|own| own.boot_id() != process.boot_id())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(boot_id: &str) -> OsProcessIdentity {
        OsProcessIdentity::new(42, 1000, boot_id.to_owned())
    }

    #[test]
    fn should_take_a_run_for_one_of_another_boot_when_the_boots_differ() {
        assert!(is_of_another_boot(&identity("a"), Some(&identity("b"))));
    }

    #[test]
    fn should_take_a_run_for_one_of_this_boot_when_the_boots_are_the_same() {
        assert!(!is_of_another_boot(&identity("a"), Some(&identity("a"))));
    }

    #[test]
    fn should_take_a_run_for_one_of_this_boot_when_the_os_does_not_tell_boots_apart() {
        assert!(!is_of_another_boot(&identity(""), Some(&identity(""))));
        assert!(!is_of_another_boot(&identity(""), None));
    }

    #[test]
    fn should_take_a_run_for_one_of_another_boot_when_the_launcher_does_not_know_its_own() {
        assert!(is_of_another_boot(&identity("a"), None));
    }
}
