use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    instance::{
        InstanceError, InstanceId, LaunchCancellations, app::ports::StopInstanceUseCasePort,
    },
    process::ProcessStorage,
};

/// Stops what the instance is doing to start or run the game: calls its launch off while there
/// is no game yet, or ends its game.
pub struct StopInstanceUseCase {
    cancellations: Arc<LaunchCancellations>,
    process_storage: Arc<dyn ProcessStorage>,
}

impl StopInstanceUseCase {
    pub fn new(
        cancellations: Arc<LaunchCancellations>,
        process_storage: Arc<dyn ProcessStorage>,
    ) -> Self {
        Self {
            cancellations,
            process_storage,
        }
    }

    /// A launch and a game are never there together, so the launch is asked first. Neither is
    /// found in the moment between the last stop of the launch and the start of the game: there
    /// is nothing to stop yet, and it is no error.
    pub async fn execute(&self, instance_id: InstanceId) -> Result<(), InstanceError> {
        if self.cancellations.cancel(&instance_id) {
            return Ok(());
        }

        for process in self.process_storage.list_metadata().await? {
            if *process.instance_id() == instance_id {
                self.process_storage.kill(process.uuid()).await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl StopInstanceUseCasePort for StopInstanceUseCase {
    async fn execute(&self, instance_id: InstanceId) -> Result<(), InstanceError> {
        self.execute(instance_id).await
    }
}

#[cfg(test)]
mod tests {
    use process_wrap::tokio::CommandWrap;
    use tokio::process::Command;

    use crate::features::process::{MinecraftProcessMetadata, infra::InMemoryProcessStorage};

    use super::*;

    fn id(name: &str) -> InstanceId {
        InstanceId::try_from(name).unwrap()
    }

    fn sleeping_process() -> Box<dyn process_wrap::tokio::ChildWrapper> {
        let mut command = if cfg!(windows) {
            let mut command = Command::new("ping");
            command.args(["-n", "60", "127.0.0.1"]);
            command
        } else {
            let mut command = Command::new("sleep");
            command.arg("60");
            command
        };
        command.stdout(std::process::Stdio::null());
        CommandWrap::from(command).spawn().unwrap()
    }

    fn use_case(
        cancellations: &Arc<LaunchCancellations>,
        storage: &Arc<InMemoryProcessStorage>,
    ) -> StopInstanceUseCase {
        StopInstanceUseCase::new(cancellations.clone(), storage.clone())
    }

    #[tokio::test]
    async fn should_call_the_launch_off_when_the_instance_is_launching() {
        let cancellations = Arc::new(LaunchCancellations::default());
        let storage = Arc::new(InMemoryProcessStorage::default());
        let launch = cancellations.register(&id("mine"));

        use_case(&cancellations, &storage)
            .execute(id("mine"))
            .await
            .unwrap();

        assert!(launch.token.is_cancelled());
    }

    #[tokio::test]
    async fn should_kill_the_game_of_the_instance_only() {
        let cancellations = Arc::new(LaunchCancellations::default());
        let storage = Arc::new(InMemoryProcessStorage::default());
        let mine = MinecraftProcessMetadata::new(id("mine"));
        let other = MinecraftProcessMetadata::new(id("other"));
        for metadata in [&mine, &other] {
            storage
                .insert(metadata.clone(), sleeping_process())
                .await
                .unwrap();
        }

        use_case(&cancellations, &storage)
            .execute(id("mine"))
            .await
            .unwrap();

        assert!(storage.was_killed(mine.uuid()).await);
        assert!(!storage.was_killed(other.uuid()).await);
        storage.kill(other.uuid()).await.unwrap();
    }

    #[tokio::test]
    async fn should_find_nothing_to_stop_without_error() {
        let cancellations = Arc::new(LaunchCancellations::default());
        let storage = Arc::new(InMemoryProcessStorage::default());

        let result = use_case(&cancellations, &storage).execute(id("mine")).await;

        assert!(result.is_ok());
    }
}
