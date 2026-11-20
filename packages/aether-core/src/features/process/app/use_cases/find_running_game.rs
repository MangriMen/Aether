use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::core::domain::error_chain;
use crate::features::{
    game_log::ListGameLogSourcesUseCasePort,
    instance::{InstanceId, InstanceStorage},
    process::{
        ProcessError, ProcessProbe, ProcessRecordStorage, ProcessStorage, RunningGameService,
    },
};

use super::{SkippedHookNotices, ended_runs::EndedRuns};

/// Finds the game of an instance, also one that only a saved record tells of: a game another
/// launcher on the same data follows, or one this launcher has not taken over yet.
pub struct FindRunningGameUseCase {
    ended_runs: EndedRuns,
}

impl FindRunningGameUseCase {
    pub fn new(
        process_storage: Arc<dyn ProcessStorage>,
        records: Arc<dyn ProcessRecordStorage>,
        probe: Arc<dyn ProcessProbe>,
        instance_storage: Arc<dyn InstanceStorage>,
        game_logs: Arc<dyn ListGameLogSourcesUseCasePort>,
        notices: Arc<SkippedHookNotices>,
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

        // The records only add to what the launcher follows: when they cannot be read, the
        // launch goes on as it would without them.
        let records = match runs.records.list_for_instance(instance_id).await {
            Ok(records) => records,
            Err(err) => {
                tracing::warn!(
                    %instance_id,
                    error = %error_chain(&err),
                    "Could not read the saved games of the instance"
                );
                return Ok(None);
            }
        };
        for record in records {
            match runs.probe.is_alive(&record.process).await {
                Ok(true) => return Ok(Some(record.id)),
                Ok(false) => {
                    if let Err(err) = runs.forget(&record).await {
                        tracing::warn!(
                            process_id = %record.id,
                            %instance_id,
                            error = %error_chain(&err),
                            "Could not forget the saved game that has ended"
                        );
                    }
                }
                // The game may still run: launching a second one is worse than refusing.
                Err(err) => {
                    tracing::warn!(
                        process_id = %record.id,
                        %instance_id,
                        error = %err,
                        "Could not tell whether the saved game still runs: it is taken for running"
                    );
                    return Ok(Some(record.id));
                }
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl RunningGameService for FindRunningGameUseCase {
    async fn running_game(&self, instance_id: &InstanceId) -> Result<Option<Uuid>, ProcessError> {
        self.execute(instance_id).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::process::{
        MinecraftProcessMetadata, ProcessHandle,
        app::ports::TakeSkippedHookNoticesUseCasePort,
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
    }

    #[tokio::test]
    async fn should_find_the_saved_game_when_it_still_runs() {
        let record = record_of(10, earlier_launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        world.probe.set(10, Look::Running);

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, Some(record.id));
        assert!(world.records.get(record.id).is_some());
    }

    #[tokio::test]
    async fn should_take_the_saved_game_for_running_when_the_os_cannot_tell() {
        let record = record_of(10, earlier_launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        world.probe.set(10, Look::Unknown);

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, Some(record.id));
        assert!(world.records.get(record.id).is_some());
    }

    #[tokio::test]
    async fn should_forget_the_saved_game_as_the_start_does_when_it_is_gone() {
        let record = record_of(10, earlier_launcher());
        let world = World::new(std::slice::from_ref(&record), Some(launcher()));
        *world.game_logs.0.lock().unwrap() =
            Some(record.playtime_credited_until + Duration::seconds(120));

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, None);
        assert!(world.records.get(record.id).is_none());
        assert_eq!(world.instances.time_played(), 120);
        assert_eq!(world.notices.execute().await.len(), 1);
    }

    #[tokio::test]
    async fn should_find_no_game_when_the_saved_games_cannot_be_read() {
        let world = World::new(&[record_of(10, earlier_launcher())], Some(launcher()));
        world.probe.set(10, Look::Running);
        *world.records.failing_reads.lock().unwrap() = true;

        let running = use_case(&world).execute(&instance_id()).await.unwrap();

        assert_eq!(running, None);
    }
}
