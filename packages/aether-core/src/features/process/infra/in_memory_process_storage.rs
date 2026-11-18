use std::process::ExitStatus;

use async_trait::async_trait;
use dashmap::DashMap;
use process_wrap::tokio::ChildWrapper;
use uuid::Uuid;

use super::super::app::ProcessStorage;
use super::super::domain::{MinecraftProcessMetadata, ProcessError};

#[derive(Debug)]
struct ActiveProcess {
    metadata: MinecraftProcessMetadata,
    child: Box<dyn ChildWrapper>,
    killed: bool,
}

#[derive(Debug, Default)]
pub struct InMemoryProcessStorage {
    processes: DashMap<Uuid, ActiveProcess>,
}

#[async_trait]
impl ProcessStorage for InMemoryProcessStorage {
    async fn insert(
        &self,
        metadata: MinecraftProcessMetadata,
        child: Box<dyn ChildWrapper>,
    ) -> Result<(), ProcessError> {
        self.processes.insert(
            metadata.uuid(),
            ActiveProcess {
                metadata,
                child,
                killed: false,
            },
        );

        Ok(())
    }

    async fn remove(&self, id: Uuid) -> Result<(), ProcessError> {
        self.processes.remove(&id);
        Ok(())
    }

    async fn list_metadata(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError> {
        Ok(self.processes.iter().map(|x| x.metadata.clone()).collect())
    }

    async fn get_metadata(
        &self,
        id: Uuid,
    ) -> Result<Option<MinecraftProcessMetadata>, ProcessError> {
        Ok(self.processes.get(&id).map(|x| x.metadata.clone()))
    }

    async fn try_wait(&self, id: Uuid) -> Result<Option<Option<ExitStatus>>, ProcessError> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            return Ok(Some(process.child.try_wait().map_err(|err| {
                tracing::warn!(process_id = %id, "Failed to wait for the game process: {err}");
                ProcessError::WaitError { id: id.to_string() }
            })?));
        }
        Ok(None)
    }

    async fn kill(&self, id: Uuid) -> Result<(), ProcessError> {
        // Only signalled under the entry, never awaited: the tracker and the process list wait
        // on the same shard, and a game that is slow to close would hold them all.
        let Some(mut process) = self.processes.get_mut(&id) else {
            return Ok(());
        };
        // A second stop while the first one takes effect: the process may already be reaped,
        // and stopping it again would report a failure.
        if process.killed {
            return Ok(());
        }
        if let Err(err) = process.child.start_kill() {
            tracing::warn!(process_id = %id, "Failed to kill the game process: {err}");
            return Err(ProcessError::KillError { id: id.to_string() });
        }
        process.killed = true;
        Ok(())
    }

    async fn was_killed(&self, id: Uuid) -> bool {
        self.processes.get(&id).is_some_and(|p| p.killed)
    }
}

#[cfg(test)]
mod tests {
    use std::{future::Future, pin::Pin, time::Duration};

    use crate::features::instance::InstanceId;

    use super::*;

    /// A process that takes the stop signal but never exits.
    #[derive(Debug)]
    struct NeverExits;

    impl ChildWrapper for NeverExits {
        fn inner(&self) -> &dyn ChildWrapper {
            unreachable!("there is no wrapped child")
        }
        fn inner_mut(&mut self) -> &mut dyn ChildWrapper {
            unreachable!("there is no wrapped child")
        }
        fn into_inner(self: Box<Self>) -> Box<dyn ChildWrapper> {
            unreachable!("there is no wrapped child")
        }
        fn start_kill(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
            Ok(None)
        }
        fn wait(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = std::io::Result<ExitStatus>> + Send + '_>> {
            Box::pin(std::future::pending())
        }
    }

    async fn storage_with_a_process_that_never_exits() -> (InMemoryProcessStorage, Uuid) {
        let storage = InMemoryProcessStorage::default();
        let metadata = MinecraftProcessMetadata::new(InstanceId::generate());
        let id = metadata.uuid();
        storage
            .insert(metadata, Box::new(NeverExits))
            .await
            .unwrap();
        (storage, id)
    }

    /// `kill`, which must not wait for the exit: with [`NeverExits`] that wait would never end.
    async fn kill_without_waiting(
        storage: &InMemoryProcessStorage,
        id: Uuid,
    ) -> Result<(), ProcessError> {
        tokio::time::timeout(Duration::from_secs(10), storage.kill(id))
            .await
            .expect("kill waited for the process to exit")
    }

    #[tokio::test]
    async fn should_keep_the_process_list_readable_when_a_killed_process_has_not_exited() {
        let (storage, id) = storage_with_a_process_that_never_exits().await;

        kill_without_waiting(&storage, id).await.unwrap();

        assert_eq!(storage.list_metadata().await.unwrap().len(), 1);
        assert_eq!(storage.try_wait(id).await.unwrap(), Some(None));
        assert!(storage.was_killed(id).await);
    }

    #[tokio::test]
    async fn should_accept_a_second_kill_when_the_first_one_is_taking_effect() {
        let (storage, id) = storage_with_a_process_that_never_exits().await;

        kill_without_waiting(&storage, id).await.unwrap();

        assert!(kill_without_waiting(&storage, id).await.is_ok());
    }
}
