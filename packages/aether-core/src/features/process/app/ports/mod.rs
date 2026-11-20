mod manage_process_service;
mod process_probe;
mod process_record_storage;
mod process_start_service;
mod running_game_service;
mod skipped_hook_notice_service;
mod track_process_service;

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use process_wrap::tokio::ChildWrapper;
use uuid::Uuid;

use crate::features::events::WarningEvent;
use crate::features::process::domain::{MinecraftProcessMetadata, ProcessError, ProcessState};

pub use manage_process_service::ManageProcessService;
pub use process_probe::{AdoptedProcess, ProcessProbe};
pub use process_record_storage::ProcessRecordStorage;
pub use process_start_service::ProcessStartService;
pub use running_game_service::RunningGameService;
pub use skipped_hook_notice_service::SkippedHookNoticeService;
pub use track_process_service::{TrackProcessService, TrackedEnd};

/// The work that takes over, or forgets, the games shown as pending. It ends once every one is
/// settled, which for a game another running launcher follows may be never.
pub type PendingAdoptions = Pin<Box<dyn Future<Output = ()> + Send>>;

/// The game process as the launcher holds it.
#[derive(Debug)]
pub enum ProcessHandle {
    /// A child of this launcher.
    Owned(Box<dyn ChildWrapper>),
    /// A game an earlier launcher started.
    Adopted(Box<dyn AdoptedProcess>),
    /// A game an earlier launcher started, which this one has not taken over yet: it counts as
    /// running, and stopping it is refused until it is taken over.
    Pending,
}

#[async_trait]
pub trait ProcessStorage: Send + Sync {
    async fn insert(
        &self,
        process: MinecraftProcessMetadata,
        handle: ProcessHandle,
    ) -> Result<(), ProcessError>;
    async fn remove(&self, id: Uuid) -> Result<(), ProcessError>;
    /// Puts `handle` in the place of the [`ProcessHandle::Pending`] one of `id`; false when `id`
    /// is not pending in the storage.
    async fn replace_pending(&self, id: Uuid, handle: ProcessHandle) -> Result<bool, ProcessError>;

    async fn list_metadata(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError>;
    async fn get_metadata(
        &self,
        id: Uuid,
    ) -> Result<Option<MinecraftProcessMetadata>, ProcessError>;

    /// `None` when the storage does not hold the process.
    async fn try_wait(&self, id: Uuid) -> Result<Option<ProcessState>, ProcessError>;
    /// Ask the process to stop and return without waiting for it: the exit is seen by `try_wait`.
    async fn kill(&self, id: Uuid) -> Result<(), ProcessError>;
    /// Whether `kill` was requested for the process (the exit is then not a crash).
    async fn was_killed(&self, id: Uuid) -> bool;
}

// ── Use case ports ──

#[async_trait]
pub trait KillProcessUseCasePort: Send + Sync {
    async fn execute(&self, process_id: Uuid) -> Result<(), ProcessError>;
}

#[async_trait]
pub trait ListProcessMetadataUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError>;
}

#[async_trait]
pub trait AdoptGameProcessesUseCasePort: Send + Sync {
    /// Shows every game an earlier launcher left running as running, before it returns. The
    /// games are taken over, or forgotten when they are gone, by the work it hands back, which
    /// is meant to run in the background.
    async fn execute(&self) -> PendingAdoptions;
}

#[async_trait]
pub trait TakeSkippedHookNoticesUseCasePort: Send + Sync {
    /// The notices of skipped post-exit hooks held since the launcher started; once taken,
    /// later ones are sent as events.
    async fn execute(&self) -> Vec<WarningEvent>;
}
