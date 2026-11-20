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
use chrono::Utc;
use tokio::task::JoinSet;

use crate::core::domain::error_chain;
use crate::features::{
    events::{EventEmitterExt, ProcessEvent, ProcessEventType, SharedEventEmitter},
    game_log::ListGameLogSourcesUseCasePort,
    instance::InstanceStorage,
    process::{
        AdoptedProcess, ManageProcessService, MinecraftProcessMetadata, OsProcessIdentity,
        ProcessHandle, ProcessOrigin, ProcessProbe, ProcessRecord, ProcessRecordStorage,
        ProcessStorage, app::ports::AdoptGameProcessesUseCasePort,
    },
};

use super::{ManageProcessParams, SkippedHookNotices, ended_runs::EndedRuns};

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
        notices: Arc<SkippedHookNotices>,
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
        PendingAdoptions {
            adoption: adoption.clone(),
            records: pending,
        }
    }
}

#[async_trait]
impl AdoptGameProcessesUseCasePort for AdoptGameProcessesUseCase {
    async fn execute(&self) -> PendingAdoptions {
        self.execute().await
    }
}

/// The games shown as running that are still to be taken over or forgotten.
pub struct PendingAdoptions {
    adoption: Arc<Adoption>,
    records: Vec<ProcessRecord>,
}

impl PendingAdoptions {
    /// Takes over or forgets every game. Returns once each one is settled, which for a game
    /// another running launcher follows may be never.
    pub async fn resolve(self) {
        let mut games = JoinSet::new();
        for record in self.records {
            games.spawn(self.adoption.clone().resolve(record));
        }
        while let Some(result) = games.join_next().await {
            if let Err(err) = result {
                tracing::error!(error = %err, "Taking over a game failed");
            }
        }
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
    async fn resolve(self: Arc<Self>, mut record: ProcessRecord) {
        let mut retry = self.pace.retry_min;
        let mut first = true;
        loop {
            let step = self.try_take_over(&record, first).await;
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
            // Another launcher may have taken the game over or ended its run meanwhile.
            match self.reread(&record).await {
                Ok(Some(fresh)) => record = fresh,
                Ok(None) => {
                    tracing::debug!(
                        process_id = %record.id,
                        "The run of the game was ended by another launcher"
                    );
                    self.drop_pending(&record).await;
                    return;
                }
                Err(()) => {}
            }
        }
    }

    async fn try_take_over(&self, record: &ProcessRecord, wait_for_owner: bool) -> Step {
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
        }

        match self.probe.adopt(record).await {
            Ok(Some(process)) => self.take_over(record, process).await,
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

    async fn take_over(&self, record: &ProcessRecord, process: Box<dyn AdoptedProcess>) -> Step {
        let Some(own) = self.probe.own_identity() else {
            // The owner of the record cannot be changed to a launcher that is not known: the
            // game stays shown as running until it ends.
            tracing::warn!(
                process_id = %record.id,
                "Could not take over the game: the launcher does not know its own process"
            );
            return Step::Unknown;
        };
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

        self.credit_unfollowed_time(record).await;
        self.follow(record, process).await;
        Step::Settled
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

    async fn follow(&self, record: &ProcessRecord, process: Box<dyn AdoptedProcess>) {
        let handle = ProcessHandle::Adopted(process);
        let replaced = match self
            .process_storage
            .replace_pending(record.id, handle)
            .await
        {
            Ok(replaced) => replaced,
            Err(err) => {
                tracing::error!(
                    process_id = %record.id,
                    error = %error_chain(&err),
                    "Could not follow the game that was taken over"
                );
                return;
            }
        };
        if !replaced {
            tracing::error!(
                process_id = %record.id,
                "The game that was taken over is no longer shown"
            );
            return;
        }

        let manage_process_service = self.manage_process_service.clone();
        let params = ManageProcessParams {
            process_id: record.id,
            instance_id: record.instance_id.clone(),
            post_exit_command: record.post_exit_command.clone(),
            capture: None,
            origin: ProcessOrigin::Adopted,
        };
        tokio::spawn(async move {
            let process_id = params.process_id;
            if let Err(err) = manage_process_service.execute(params).await {
                tracing::error!(%process_id, "Could not follow the game process: {err}");
            }
        });

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
