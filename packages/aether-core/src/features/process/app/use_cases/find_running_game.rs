use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::core::domain::error_chain;
use crate::features::{
    game_log::ListGameLogSourcesUseCasePort,
    instance::{InstanceId, InstanceStorage},
    process::{
        KillScope, ProcessError, ProcessProbe, ProcessRecord, ProcessRecordStorage, ProcessStorage,
        RunningGameService, app::ports::SkippedHookNoticeService,
    },
};

use super::ended_runs::EndedRuns;

/// Finds the game of an instance, also one that only a saved record tells of: a game another
/// launcher on the same data follows, or one this launcher has not taken over yet.
pub struct FindRunningGameUseCase {
    ended_runs: EndedRuns,
}

/// What a saved record tells of its game.
enum Saved {
    Running,
    Ended,
}

impl FindRunningGameUseCase {
    pub fn new(
        process_storage: Arc<dyn ProcessStorage>,
        records: Arc<dyn ProcessRecordStorage>,
        probe: Arc<dyn ProcessProbe>,
        instance_storage: Arc<dyn InstanceStorage>,
        game_logs: Arc<dyn ListGameLogSourcesUseCasePort>,
        notices: Arc<dyn SkippedHookNoticeService>,
    ) -> Self {
        Self {
            ended_runs: EndedRuns {
                records,
                process_storage,
                instance_storage,
                game_logs,
                probe,
                notices,
            },
        }
    }

    pub async fn execute(&self, instance_id: &InstanceId) -> Result<Option<Uuid>, ProcessError> {
        let runs = &self.ended_runs;
        if let Some(process) = runs
            .process_storage
            .list_metadata()
            .await?
            .into_iter()
            .find(|process| process.instance_id() == instance_id)
        {
            return Ok(Some(process.uuid()));
        }

        for record in self.saved(Some(instance_id)).await? {
            match self.look_at(&record).await {
                Saved::Running => return Ok(Some(record.id)),
                Saved::Ended => {
                    if let Err(err) = runs.forget(&record).await {
                        tracing::warn!(
                            process_id = %record.id,
                            %instance_id,
                            error = %error_chain(&err),
                            "Could not forget the saved game that has ended"
                        );
                    }
                }
            }
        }
        Ok(None)
    }

    /// The saved records, of one instance or of all. Unreadable ones are an error: one of them
    /// may stand for a game that runs.
    async fn saved(
        &self,
        instance_id: Option<&InstanceId>,
    ) -> Result<Vec<ProcessRecord>, ProcessError> {
        let records = &self.ended_runs.records;
        let saved = match instance_id {
            Some(instance_id) => records.list_for_instance(instance_id).await,
            None => records.list().await,
        };
        saved.map_err(|err| {
            tracing::warn!(
                instance_id = instance_id.map(tracing::field::display),
                error = %error_chain(&err),
                "Could not read the saved games: a game is taken for running"
            );
            ProcessError::SavedGamesUnreadable(err)
        })
    }

    /// Whether the game of the record carries on across a restart of the launcher. Stopping a
    /// game behind a wrapper after the restart ends the wrapper alone, so it is not left to.
    fn carries_on(&self, record: &ProcessRecord) -> bool {
        record.kill_scope == KillScope::Process && self.ended_runs.probe.tells_exit_of_adopted()
    }

    /// A game the OS cannot tell about may still run, and launching a second one is worse than
    /// refusing. A game that ended while another launcher still follows it is left to that
    /// launcher, which credits and forgets it.
    async fn look_at(&self, record: &ProcessRecord) -> Saved {
        let probe = &self.ended_runs.probe;
        match probe.is_alive(&record.process).await {
            Ok(true) => return Saved::Running,
            Ok(false) => {}
            Err(err) => {
                tracing::warn!(
                    process_id = %record.id,
                    error = %err,
                    "Could not tell whether the saved game still runs: it is taken for running"
                );
                return Saved::Running;
            }
        }
        if probe.own_identity() == Some(&record.owner) {
            return Saved::Ended;
        }
        match probe.is_alive(&record.owner).await {
            Ok(false) => Saved::Ended,
            Ok(true) => Saved::Running,
            Err(err) => {
                tracing::warn!(
                    process_id = %record.id,
                    owner_pid = record.owner.pid(),
                    error = %err,
                    "Could not tell whether the launcher that follows the saved game still runs: the game is taken for running"
                );
                Saved::Running
            }
        }
    }
}

#[async_trait]
impl RunningGameService for FindRunningGameUseCase {
    async fn running_games_carry_on(&self) -> Result<bool, ProcessError> {
        let runs = &self.ended_runs;
        // Both the saved records and the games of the launcher: a game it follows but has not
        // saved is not known to carry on.
        let saved = self.saved(None).await?;
        for process in runs.process_storage.list_metadata().await? {
            let record = saved.iter().find(|record| record.id == process.uuid());
            if !record.is_some_and(|record| self.carries_on(record)) {
                return Ok(false);
            }
        }
        for record in &saved {
            if !self.carries_on(record) && matches!(self.look_at(record).await, Saved::Running) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn running_game(&self, instance_id: &InstanceId) -> Result<Option<Uuid>, ProcessError> {
        self.execute(instance_id).await
    }

    async fn any_running(&self) -> Result<bool, ProcessError> {
        if !self
            .ended_runs
            .process_storage
            .list_metadata()
            .await?
            .is_empty()
        {
            return Ok(true);
        }
        for record in self.saved(None).await? {
            if matches!(self.look_at(&record).await, Saved::Running) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::process::{
        KillScope, MinecraftProcessMetadata, ProcessHandle,
        app::use_cases::fakes::{Look, World, earlier_launcher, instance_id, launcher, record_of},
    };

    fn use_case(world: &World) -> FindRunningGameUseCase {
        FindRunningGameUseCase::new(
            world.storage.clone(),
            world.records.clone(),
            world.probe.clone(),
            world.instances.clone(),
            world.game_logs.clone(),
            world.notices.clone(),
        )
    }

    #[tokio::test]
    async fn should_find_the_game_the_launcher_follows_without_looking_at_the_records() {
        let world = World::new(&[], Some(launcher()));
        let metadata = MinecraftProcessMetadata::new(instance_id());
        let id = metadata.uuid();
        world
            .storage
            .insert(metadata, ProcessHandle::Pending)
            .await
            .unwrap();
        *world.records.failing_reads.lock().unwrap() = true;

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, Some(id));
        assert!(use_case(&world).any_running().await.unwrap());
    }

    #[tokio::test]
    async fn should_find_the_saved_game_when_it_still_runs() {
        let record = record_of(10, earlier_launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        world.probe.set(10, Look::Running);

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, Some(record.id));
        assert!(world.records.get(record.id).is_some());
        assert!(use_case(&world).any_running().await.unwrap());
    }

    #[tokio::test]
    async fn should_take_the_saved_game_for_running_when_the_os_cannot_tell() {
        let record = record_of(10, earlier_launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        world.probe.set(10, Look::Unknown);

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, Some(record.id));
        assert!(world.records.get(record.id).is_some());
        assert!(use_case(&world).any_running().await.unwrap());
    }

    #[tokio::test]
    async fn should_forget_the_saved_game_as_the_start_does_when_it_is_gone() {
        let record = record_of(10, earlier_launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        *world.game_logs.0.lock().unwrap() =
            Some(record.playtime_credited_until + Duration::seconds(120));

        assert!(!use_case(&world).any_running().await.unwrap());
        assert!(world.records.get(record.id).is_some(), "looking forgot it");
        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, None);
        assert!(world.records.get(record.id).is_none());
        assert_eq!(world.instances.time_played(), 120);
        assert_eq!(world.notices.execute().len(), 1);
    }

    #[rstest::rstest]
    #[case::launcher_runs(Look::Running)]
    #[case::launcher_unknown(Look::Unknown)]
    #[tokio::test]
    async fn should_leave_the_ended_game_to_the_launcher_that_follows_it_when_that_one_may_run(
        #[case] launcher_look: Look,
    ) {
        let record = record_of(10, earlier_launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        world.probe.set(2, launcher_look);

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, Some(record.id));
        assert!(world.records.get(record.id).is_some());
        assert_eq!(world.instances.time_played(), 0);
        assert!(use_case(&world).any_running().await.unwrap());
    }

    #[tokio::test]
    async fn should_forget_the_ended_game_of_this_launcher_without_looking_at_the_launcher() {
        let record = record_of(10, launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, None);
        assert!(world.records.get(record.id).is_none());
        assert_eq!(world.probe.times_asked(1), 0);
    }

    #[tokio::test]
    async fn should_carry_on_when_no_game_runs() {
        let world = World::new(&[], Some(launcher()));

        assert!(use_case(&world).running_games_carry_on().await.unwrap());
    }

    async fn follow(world: &World, record: &ProcessRecord) {
        let metadata = MinecraftProcessMetadata::adopted(
            record.id,
            record.instance_id.clone(),
            record.started_at,
        );
        world
            .storage
            .insert(metadata, ProcessHandle::Pending)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_carry_on_when_the_game_has_a_record_of_a_started_process() {
        let record = record_of(10, launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        follow(&world, &record).await;

        assert!(use_case(&world).running_games_carry_on().await.unwrap());
    }

    #[tokio::test]
    async fn should_not_carry_on_when_the_followed_game_has_no_record() {
        let world = World::new(&[], Some(launcher()));
        let metadata = MinecraftProcessMetadata::new(instance_id());
        world
            .storage
            .insert(metadata, ProcessHandle::Pending)
            .await
            .unwrap();

        assert!(!use_case(&world).running_games_carry_on().await.unwrap());
    }

    #[tokio::test]
    async fn should_not_carry_on_when_the_game_is_behind_a_wrapper() {
        let mut record = record_of(10, launcher());
        record.kill_scope = KillScope::Tree;
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        follow(&world, &record).await;

        assert!(!use_case(&world).running_games_carry_on().await.unwrap());
    }

    #[tokio::test]
    async fn should_not_carry_on_when_the_os_does_not_tell_how_an_adopted_game_ended() {
        let record = record_of(10, launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        follow(&world, &record).await;
        world.probe.tell_exit(false);

        assert!(!use_case(&world).running_games_carry_on().await.unwrap());
    }

    #[tokio::test]
    async fn should_not_carry_on_when_a_game_of_another_launcher_would_not() {
        let mut record = record_of(10, earlier_launcher());
        record.kill_scope = KillScope::Tree;
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        world.probe.set(10, Look::Running);
        world.probe.set(2, Look::Running);

        assert!(!use_case(&world).running_games_carry_on().await.unwrap());
    }

    #[tokio::test]
    async fn should_not_mind_a_game_that_is_gone_when_it_would_not_carry_on() {
        let mut record = record_of(10, launcher());
        record.kill_scope = KillScope::Tree;
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));

        assert!(use_case(&world).running_games_carry_on().await.unwrap());
    }

    #[tokio::test]
    async fn should_refuse_when_the_saved_games_cannot_be_read() {
        let world = World::new(&[record_of(10, earlier_launcher())], Some(launcher()));
        *world.records.failing_reads.lock().unwrap() = true;

        let running = use_case(&world).execute(&instance_id()).await;
        let any = use_case(&world).any_running().await;

        assert!(
            matches!(running, Err(ProcessError::SavedGamesUnreadable(_))),
            "{running:?}"
        );
        assert!(matches!(any, Err(ProcessError::SavedGamesUnreadable(_))));
    }
}
