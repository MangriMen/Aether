//! Takes over the games an earlier launcher left running.
//!
//! Every saved game is shown as running before the launcher takes any command, so that it is
//! not launched twice, removed or updated under the game while the launcher looks at it. Each
//! is then taken over, or forgotten once known to be gone, on its own: one that cannot be
//! looked at is tried again later and never forgotten, and one that another running launcher
//! follows is left to it until that launcher is gone.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::task::JoinSet;

use crate::core::domain::error_chain;
use crate::features::{
    events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
    game_log::{GameLogTailService, ListGameLogSourcesUseCasePort},
    instance::InstanceStorage,
    process::{
        AdoptedProcess, ManageProcessService, MinecraftProcessMetadata, OsProcessIdentity,
        ProcessHandle, ProcessOrigin, ProcessProbe, ProcessRecord, ProcessRecordStorage,
        ProcessStorage,
        app::ports::{AdoptGameProcessesUseCasePort, PendingAdoptions, SkippedHookNoticeService},
    },
};

use super::{CrashWatch, ManageProcessParams, ended_runs::EndedRuns};

/// How long the launcher that followed a game may take to exit before the game is left to it:
/// a restart for an update starts the new launcher before the old one has quit.
const ADOPTION_OWNER_WAIT: Duration = Duration::from_secs(10);

/// How the taking over is paced.
#[derive(Debug, Clone, Copy)]
struct Pace {
    owner_wait: Duration,
    owner_poll: Duration,
    /// The first wait before a game the OS could not tell about is looked at again; it doubles
    /// up to `retry_max`.
    retry_min: Duration,
    retry_max: Duration,
    /// How often a game another launcher follows is looked at again.
    foreign_recheck: Duration,
}

impl Default for Pace {
    fn default() -> Self {
        Self {
            owner_wait: ADOPTION_OWNER_WAIT,
            owner_poll: Duration::from_millis(100),
            retry_min: Duration::from_secs(1),
            retry_max: Duration::from_secs(60),
            foreign_recheck: Duration::from_secs(5),
        }
    }
}

struct Adoption {
    process_storage: Arc<dyn ProcessStorage>,
    records: Arc<dyn ProcessRecordStorage>,
    probe: Arc<dyn ProcessProbe>,
    manage_process_service: Arc<dyn ManageProcessService>,
    event_emitter: SharedEventEmitter,
    log_tails: Arc<dyn GameLogTailService>,
    ended_runs: EndedRuns,
    pace: Pace,
}

pub struct AdoptGameProcessesUseCase {
    adoption: Arc<Adoption>,
}

impl AdoptGameProcessesUseCase {
    #[expect(
        clippy::too_many_arguments,
        reason = "one collaborator per argument, wired once in the DI container"
    )]
    pub fn new(
        process_storage: Arc<dyn ProcessStorage>,
        records: Arc<dyn ProcessRecordStorage>,
        probe: Arc<dyn ProcessProbe>,
        manage_process_service: Arc<dyn ManageProcessService>,
        event_emitter: SharedEventEmitter,
        instance_storage: Arc<dyn InstanceStorage>,
        game_logs: Arc<dyn ListGameLogSourcesUseCasePort>,
        log_tails: Arc<dyn GameLogTailService>,
        notices: Arc<dyn SkippedHookNoticeService>,
    ) -> Self {
        Self {
            adoption: Arc::new(Adoption {
                ended_runs: EndedRuns {
                    records: records.clone(),
                    process_storage: process_storage.clone(),
                    instance_storage,
                    game_logs,
                    probe: probe.clone(),
                    notices,
                },
                process_storage,
                records,
                probe,
                manage_process_service,
                event_emitter,
                log_tails,
                pace: Pace::default(),
            }),
        }
    }

    #[cfg(test)]
    fn with_pace(mut self, pace: Pace) -> Self {
        Arc::get_mut(&mut self.adoption)
            .expect("the use case has not handed out its work yet")
            .pace = pace;
        self
    }

    pub async fn execute(&self) -> PendingAdoptions {
        let adoption = &self.adoption;
        let records = adoption.records.list().await.unwrap_or_else(|err| {
            tracing::warn!(
                error = %error_chain(&err),
                "Could not read the games an earlier launcher left running"
            );
            Vec::new()
        });
        let mut pending = Vec::with_capacity(records.len());
        for record in records {
            let metadata = MinecraftProcessMetadata::adopted(
                record.id,
                record.instance_id.clone(),
                record.started_at,
            );
            match adoption
                .process_storage
                .insert(metadata, ProcessHandle::Pending)
                .await
            {
                Ok(()) => pending.push(record),
                Err(err) => tracing::warn!(
                    process_id = %record.id,
                    instance_id = %record.instance_id,
                    error = %error_chain(&err),
                    "Could not show the game an earlier launcher left running"
                ),
            }
        }
        Box::pin(adoption.clone().resolve_all(pending))
    }
}

#[async_trait]
impl AdoptGameProcessesUseCasePort for AdoptGameProcessesUseCase {
    async fn execute(&self) -> PendingAdoptions {
        self.execute().await
    }
}

/// Where a try to take a game over left it.
enum Step {
    Settled,
    /// Another running launcher follows it.
    Foreign,
    /// It could not be told whether the game runs, or it could not be taken over.
    Unknown,
}

impl Adoption {
    async fn resolve_all(self: Arc<Self>, records: Vec<ProcessRecord>) {
        let mut games = JoinSet::new();
        for record in records {
            games.spawn(self.clone().resolve(record));
        }
        while let Some(result) = games.join_next().await {
            if let Err(err) = result {
                tracing::error!(error = %err, "Taking over a game failed");
            }
        }
    }

    async fn resolve(self: Arc<Self>, mut record: ProcessRecord) {
        let mut retry = self.pace.retry_min;
        let mut first = true;
        loop {
            let step = self.try_take_over(&mut record, first).await;
            first = false;
            let wait = match step {
                Step::Settled => return,
                Step::Foreign => self.pace.foreign_recheck,
                Step::Unknown => {
                    let wait = retry;
                    retry = (retry * 2).min(self.pace.retry_max);
                    wait
                }
            };
            tokio::time::sleep(wait).await;
        }
    }

    /// One try at the game of `record`, which is read again on the way: another launcher may
    /// have moved its playtime mark, taken it over or ended its run since it was last read.
    async fn try_take_over(&self, record: &mut ProcessRecord, wait_for_owner: bool) -> Step {
        let Ok(fresh) = self.reread(record).await else {
            return Step::Unknown;
        };
        let Some(fresh) = fresh else {
            tracing::debug!(
                process_id = %record.id,
                "The run of the game was ended by another launcher"
            );
            self.drop_pending(record).await;
            return Step::Settled;
        };
        *record = fresh;

        let own = self.probe.own_identity();
        if own != Some(&record.owner) {
            match self.owner_runs(&record.owner, wait_for_owner).await {
                Ok(false) => {}
                Ok(true) => {
                    if wait_for_owner {
                        tracing::info!(
                            process_id = %record.id,
                            instance_id = %record.instance_id,
                            "The game is followed by another launcher that still runs"
                        );
                    }
                    return Step::Foreign;
                }
                Err(err) => {
                    tracing::warn!(
                        process_id = %record.id,
                        owner_pid = record.owner.pid(),
                        error = %err,
                        "Could not tell whether the launcher that followed the game still runs"
                    );
                    return Step::Unknown;
                }
            }
            // The owner may have moved the mark, or ended the run, while it was waited for.
            let checked_owner = record.owner.clone();
            match self.reread(record).await {
                Ok(Some(fresh)) => *record = fresh,
                Ok(None) => {
                    self.drop_pending(record).await;
                    return Step::Settled;
                }
                Err(()) => return Step::Unknown,
            }
            if !record.owner.matches(&checked_owner) {
                return Step::Foreign;
            }
        }

        // Taken before the game is looked at: a crash from here on is told by its trace.
        let seen_at = Utc::now();
        match self.probe.adopt(record).await {
            Ok(Some(process)) => self.take_over(record, process, seen_at).await,
            Ok(None) => {
                if let Err(err) = self.ended_runs.forget(record).await {
                    // The record stays for the next launch or start to forget.
                    tracing::warn!(
                        process_id = %record.id,
                        instance_id = %record.instance_id,
                        error = %error_chain(&err),
                        "Could not forget the game that has ended"
                    );
                }
                self.drop_pending(record).await;
                Step::Settled
            }
            Err(err) => {
                tracing::warn!(
                    process_id = %record.id,
                    instance_id = %record.instance_id,
                    pid = record.process.pid(),
                    error = %err,
                    "Could not tell whether the game an earlier launcher left still runs"
                );
                Step::Unknown
            }
        }
    }

    /// Whether the launcher `owner` runs: waited for up to the owner wait when `wait`, looked at
    /// once otherwise.
    async fn owner_runs(&self, owner: &OsProcessIdentity, wait: bool) -> std::io::Result<bool> {
        let deadline = tokio::time::Instant::now() + self.pace.owner_wait;
        loop {
            if !self.probe.is_alive(owner).await? {
                return Ok(false);
            }
            if !wait || tokio::time::Instant::now() >= deadline {
                return Ok(true);
            }
            tokio::time::sleep(self.pace.owner_poll).await;
        }
    }

    async fn take_over(
        &self,
        record: &ProcessRecord,
        process: Box<dyn AdoptedProcess>,
        seen_at: DateTime<Utc>,
    ) -> Step {
        let Some(own) = self.probe.own_identity() else {
            // The owner of the record cannot be changed to a launcher that is not known: the
            // game stays shown as running until it ends.
            tracing::warn!(
                process_id = %record.id,
                "Could not take over the game: the launcher does not know its own process"
            );
            return Step::Unknown;
        };
        // Looked at before the owner changes: a record this launcher owns must be followed.
        match self.process_storage.get_metadata(record.id).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                tracing::error!(
                    process_id = %record.id,
                    "The game to take over is no longer shown: it is left alone"
                );
                return Step::Settled;
            }
            Err(err) => {
                tracing::warn!(
                    process_id = %record.id,
                    error = %error_chain(&err),
                    "Could not look at the game to take over"
                );
                return Step::Unknown;
            }
        }
        match self.records.take_over(record.id, &record.owner, own).await {
            Ok(true) => {}
            Ok(false) => {
                tracing::info!(
                    process_id = %record.id,
                    instance_id = %record.instance_id,
                    "Another launcher took over the game first"
                );
                return Step::Foreign;
            }
            Err(err) => {
                tracing::warn!(
                    process_id = %record.id,
                    error = %error_chain(&err),
                    "Could not take over the game"
                );
                return Step::Unknown;
            }
        }

        let replaced = self
            .process_storage
            .replace_pending(record.id, ProcessHandle::Adopted(process))
            .await;
        if !matches!(replaced, Ok(true)) {
            let why = match &replaced {
                Err(err) => error_chain(err),
                Ok(_) => "the game is no longer pending".to_owned(),
            };
            tracing::warn!(
                process_id = %record.id,
                error = %why,
                "Could not follow the game that was taken over: it is handed back"
            );
            self.hand_back(record, own).await;
            return match replaced {
                Err(_) => Step::Unknown,
                Ok(_) => Step::Settled,
            };
        }

        self.credit_unfollowed_time(record).await;
        self.follow(record, seen_at).await;
        Step::Settled
    }

    /// Gives the record back to the launcher it was taken from, so that a launcher that can
    /// follow the game takes it over.
    async fn hand_back(&self, record: &ProcessRecord, own: &OsProcessIdentity) {
        match self.records.take_over(record.id, own, &record.owner).await {
            Ok(true) => {}
            Ok(false) => tracing::warn!(
                process_id = %record.id,
                "Could not hand the game back: its record changed"
            ),
            Err(err) => tracing::warn!(
                process_id = %record.id,
                error = %error_chain(&err),
                "Could not hand the game back"
            ),
        }
    }

    /// The game ran all the time no launcher followed it: that time is credited, once the mark
    /// is moved so that it is not credited again.
    async fn credit_unfollowed_time(&self, record: &ProcessRecord) {
        let now = Utc::now();
        if let Err(err) = self.records.touch_playtime(record.id, now).await {
            tracing::warn!(
                process_id = %record.id,
                error = %error_chain(&err),
                "Could not save the playtime mark of the game: the time it ran unfollowed is not credited"
            );
            return;
        }
        self.ended_runs
            .credit(
                &record.instance_id,
                now.signed_duration_since(record.playtime_credited_until),
            )
            .await;
    }

    /// Told before the following starts, so that the end of a game that exits at once comes
    /// after it.
    async fn follow(&self, record: &ProcessRecord, seen_at: DateTime<Utc>) {
        let manage_process_service = self.manage_process_service.clone();
        let params = ManageProcessParams {
            process_id: record.id,
            instance_id: record.instance_id.clone(),
            post_exit_command: record.post_exit_command.clone(),
            capture: None,
            log_tail: Some(self.log_tails.follow(&record.instance_id)),
            origin: ProcessOrigin::Adopted,
            crash_watch: Some(CrashWatch {
                pid: record.process.pid(),
                since: seen_at,
            }),
        };
        tracing::info!(
            process_id = %record.id,
            instance_id = %record.instance_id,
            pid = record.process.pid(),
            "Took over the game an earlier launcher left running"
        );
        self.event_emitter
            .emit_safe(ProcessEvent {
                instance_id: record.instance_id.clone(),
                process_id: record.id,
                event: ProcessEventType::Adopted,
                message: "Adopted process".to_string(),
                exit_code: None,
                crashed: false,
                adopted: true,
            })
            .await;
        tokio::spawn(async move {
            let process_id = params.process_id;
            if let Err(err) = manage_process_service.execute(params).await {
                tracing::error!(
                    %process_id,
                    error = %error_chain(&err),
                    "Could not follow the game process"
                );
            }
        });
    }

    async fn reread(&self, record: &ProcessRecord) -> Result<Option<ProcessRecord>, ()> {
        match self.records.list_for_instance(&record.instance_id).await {
            Ok(records) => Ok(records.into_iter().find(|other| other.id == record.id)),
            Err(err) => {
                tracing::warn!(
                    process_id = %record.id,
                    error = %error_chain(&err),
                    "Could not read the saved game again"
                );
                Err(())
            }
        }
    }

    /// Stops showing a game that has ended. The window may have shown it as running already,
    /// so its end is told like that of a followed game.
    async fn drop_pending(&self, record: &ProcessRecord) {
        if let Err(err) = self.process_storage.remove(record.id).await {
            tracing::warn!(
                process_id = %record.id,
                error = %error_chain(&err),
                "Could not stop showing the game"
            );
            return;
        }
        self.event_emitter
            .emit_safe(ProcessEvent {
                instance_id: record.instance_id.clone(),
                process_id: record.id,
                event: ProcessEventType::Finished,
                message: "Exited process".to_string(),
                exit_code: None,
                crashed: false,
                adopted: true,
            })
            .await;
    }
}

#[cfg(test)]
#[path = "adopt_processes_test.rs"]
mod tests;
