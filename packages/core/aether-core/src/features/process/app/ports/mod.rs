mod manage_process_service;
mod track_process_service;

use std::process::ExitStatus;

use async_trait::async_trait;
use tokio::process::Child;
use uuid::Uuid;

use crate::features::process::domain::{MinecraftProcessMetadata, ProcessError};

pub use manage_process_service::ManageProcessService;
pub use track_process_service::TrackProcessService;

#[async_trait]
pub trait ProcessStorage: Send + Sync {
    async fn insert(
        &self,
        process: MinecraftProcessMetadata,
        child: Child,
    ) -> Result<(), ProcessError>;
    async fn remove(&self, id: Uuid) -> Result<(), ProcessError>;

    async fn list_metadata(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError>;
    async fn get_metadata(
        &self,
        id: Uuid,
    ) -> Result<Option<MinecraftProcessMetadata>, ProcessError>;

    async fn try_wait(&self, id: Uuid) -> Result<Option<Option<ExitStatus>>, ProcessError>;
    async fn wait_for(&self, id: Uuid) -> Result<(), ProcessError>;
    async fn kill(&self, id: Uuid) -> Result<(), ProcessError>;
}
