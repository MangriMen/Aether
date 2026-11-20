use std::io;
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use dashmap::DashMap;
use process_wrap::tokio::ChildWrapper;
use uuid::Uuid;

use super::super::app::{AdoptedProcess, ProcessHandle, ProcessStorage};
use super::super::domain::{MinecraftProcessMetadata, ProcessError, ProcessState};

type SharedAdopted = Arc<Mutex<Box<dyn AdoptedProcess>>>;

#[derive(Debug)]
enum Held {
    Owned(Box<dyn ChildWrapper>),
    /// Behind a lock of its own: its calls block on the OS, so they run on a blocking thread
    /// without the process list held.
    Adopted(SharedAdopted),
}

impl From<ProcessHandle> for Held {
    fn from(handle: ProcessHandle) -> Self {
        match handle {
            ProcessHandle::Owned(child) => Self::Owned(child),
            ProcessHandle::Adopted(process) => Self::Adopted(Arc::new(Mutex::new(process))),
        }
    }
}

#[derive(Debug)]
struct ActiveProcess {
    metadata: MinecraftProcessMetadata,
    handle: Held,
    killed: bool,
}

#[derive(Debug, Default)]
pub struct InMemoryProcessStorage {
    processes: DashMap<Uuid, ActiveProcess>,
}

async fn call_adopted<T: Send + 'static>(
    process: SharedAdopted,
    call: impl FnOnce(&mut dyn AdoptedProcess) -> io::Result<T> + Send + 'static,
) -> io::Result<T> {
    tokio::task::spawn_blocking(move || {
        let mut process = process.lock().unwrap_or_else(PoisonError::into_inner);
        call(process.as_mut())
    })
    .await
    .map_err(io::Error::other)?
}

fn wait_failed(id: Uuid, err: &io::Error) -> ProcessError {
    tracing::warn!(process_id = %id, "Failed to wait for the game process: {err}");
    ProcessError::WaitError { id: id.to_string() }
}

fn kill_failed(id: Uuid, err: &io::Error) -> ProcessError {
    tracing::warn!(process_id = %id, "Failed to kill the game process: {err}");
    ProcessError::KillError { id: id.to_string() }
}

#[async_trait]
impl ProcessStorage for InMemoryProcessStorage {
    async fn insert(
        &self,
        metadata: MinecraftProcessMetadata,
        handle: ProcessHandle,
    ) -> Result<(), ProcessError> {
        self.processes.insert(
            metadata.uuid(),
            ActiveProcess {
                metadata,
                handle: handle.into(),
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

    async fn try_wait(&self, id: Uuid) -> Result<Option<ProcessState>, ProcessError> {
        let adopted = match self.processes.get_mut(&id) {
            None => return Ok(None),
            Some(mut process) => match &mut process.handle {
                Held::Owned(child) => {
                    return child
                        .try_wait()
                        .map(|status| {
                            Some(status.map_or(ProcessState::Running, |status| {
                                ProcessState::Exited(Some(status))
                            }))
                        })
                        .map_err(|err| wait_failed(id, &err));
                }
                Held::Adopted(process) => process.clone(),
            },
        };
        call_adopted(adopted, |process| process.try_wait())
            .await
            .map(Some)
            .map_err(|err| wait_failed(id, &err))
    }

    async fn kill(&self, id: Uuid) -> Result<(), ProcessError> {
        // Only signalled, never awaited, and nothing slow runs under the entry: the tracker and
        // the process list wait on the same shard, and a game that is slow to close would hold
        // them all.
        let adopted = {
            let Some(mut process) = self.processes.get_mut(&id) else {
                return Ok(());
            };
            // A second stop while the first one takes effect: the process may already be
            // reaped, and stopping it again would report a failure.
            if process.killed {
                return Ok(());
            }
            match &mut process.handle {
                Held::Owned(child) => {
                    child.start_kill().map_err(|err| kill_failed(id, &err))?;
                    process.killed = true;
                    return Ok(());
                }
                Held::Adopted(adopted) => {
                    let adopted = adopted.clone();
                    // Marked before the stop, which runs with the list let go, so that a
                    // second stop meanwhile does nothing; taken back if the stop fails.
                    process.killed = true;
                    adopted
                }
            }
        };
        if let Err(err) = call_adopted(adopted, |process| process.start_kill()).await {
            if let Some(mut process) = self.processes.get_mut(&id) {
                process.killed = false;
            }
            return Err(kill_failed(id, &err));
        }
        Ok(())
    }

    async fn was_killed(&self, id: Uuid) -> bool {
        self.processes.get(&id).is_some_and(|p| p.killed)
    }
}

#[cfg(test)]
mod tests {
    use std::process::ExitStatus;
    use std::sync::{OnceLock, Weak};
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

    /// An adopted game that runs until it is stopped and then exits with an unknown status, as
    /// one does on Unix. `stops` counts the stop requests; `refuse_stop` makes them fail.
    #[derive(Debug, Default)]
    struct AdoptedGame {
        stops: Arc<Mutex<u32>>,
        refuse_stop: bool,
    }

    impl AdoptedProcess for AdoptedGame {
        fn try_wait(&mut self) -> std::io::Result<ProcessState> {
            if *self.stops.lock().unwrap() > 0 {
                Ok(ProcessState::Exited(None))
            } else {
                Ok(ProcessState::Running)
            }
        }

        fn start_kill(&mut self) -> std::io::Result<()> {
            if self.refuse_stop {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "cannot stop an elevated game",
                ));
            }
            *self.stops.lock().unwrap() += 1;
            Ok(())
        }
    }

    async fn storage_with(handle: ProcessHandle) -> (InMemoryProcessStorage, Uuid) {
        let storage = InMemoryProcessStorage::default();
        let metadata = MinecraftProcessMetadata::new(InstanceId::generate());
        let id = metadata.uuid();
        storage.insert(metadata, handle).await.unwrap();
        (storage, id)
    }

    async fn storage_with_a_process_that_never_exits() -> (InMemoryProcessStorage, Uuid) {
        storage_with(ProcessHandle::Owned(Box::new(NeverExits))).await
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
        assert_eq!(
            storage.try_wait(id).await.unwrap(),
            Some(ProcessState::Running)
        );
        assert!(storage.was_killed(id).await);
    }

    #[tokio::test]
    async fn should_accept_a_second_kill_when_the_first_one_is_taking_effect() {
        let (storage, id) = storage_with_a_process_that_never_exits().await;

        kill_without_waiting(&storage, id).await.unwrap();

        assert!(kill_without_waiting(&storage, id).await.is_ok());
    }

    #[tokio::test]
    async fn should_know_no_process_when_it_was_never_inserted() {
        let storage = InMemoryProcessStorage::default();

        assert_eq!(storage.try_wait(Uuid::new_v4()).await.unwrap(), None);
    }

    #[tokio::test]
    async fn should_follow_and_stop_an_adopted_game_like_a_started_one() {
        let game = AdoptedGame::default();
        let stops = game.stops.clone();
        let (storage, id) = storage_with(ProcessHandle::Adopted(Box::new(game))).await;

        let before = storage.try_wait(id).await.unwrap();
        kill_without_waiting(&storage, id).await.unwrap();
        kill_without_waiting(&storage, id).await.unwrap();

        assert_eq!(before, Some(ProcessState::Running));
        assert_eq!(*stops.lock().unwrap(), 1);
        assert!(storage.was_killed(id).await);
        assert_eq!(
            storage.try_wait(id).await.unwrap(),
            Some(ProcessState::Exited(None))
        );
    }

    #[tokio::test]
    async fn should_report_a_failed_stop_when_the_adopted_game_cannot_be_stopped() {
        let game = AdoptedGame {
            refuse_stop: true,
            ..AdoptedGame::default()
        };
        let (storage, id) = storage_with(ProcessHandle::Adopted(Box::new(game))).await;

        let result = kill_without_waiting(&storage, id).await;

        assert!(matches!(result, Err(ProcessError::KillError { .. })));
        assert!(!storage.was_killed(id).await);
        assert_eq!(
            storage.try_wait(id).await.unwrap(),
            Some(ProcessState::Running)
        );
    }

    /// An adopted game that notes, on every call, whether its entry in `storage` was held.
    #[derive(Debug, Default)]
    struct ChecksTheList {
        storage: Arc<OnceLock<(Weak<InMemoryProcessStorage>, Uuid)>>,
        entry_held: Arc<Mutex<Vec<bool>>>,
    }

    impl ChecksTheList {
        fn note(&self) {
            let (storage, id) = self.storage.get().expect("the storage is set");
            let storage = storage.upgrade().expect("the storage is alive");
            let held = storage.processes.try_get(id).is_locked();
            self.entry_held.lock().unwrap().push(held);
        }
    }

    impl AdoptedProcess for ChecksTheList {
        fn try_wait(&mut self) -> std::io::Result<ProcessState> {
            self.note();
            Ok(ProcessState::Running)
        }

        fn start_kill(&mut self) -> std::io::Result<()> {
            self.note();
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_let_go_of_the_process_list_while_an_adopted_game_is_looked_at() {
        let game = ChecksTheList::default();
        let target = game.storage.clone();
        let entry_held = game.entry_held.clone();
        let storage = Arc::new(InMemoryProcessStorage::default());
        let metadata = MinecraftProcessMetadata::new(InstanceId::generate());
        let id = metadata.uuid();
        storage
            .insert(metadata, ProcessHandle::Adopted(Box::new(game)))
            .await
            .unwrap();
        target.set((Arc::downgrade(&storage), id)).unwrap();

        storage.try_wait(id).await.unwrap();
        storage.kill(id).await.unwrap();

        assert_eq!(*entry_held.lock().unwrap(), [false, false]);
    }
}
