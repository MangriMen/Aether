use std::process::ExitStatus;

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::process::Child;
use uuid::Uuid;

use crate::features::process::{MinecraftProcessMetadata, ProcessError, ProcessStorage};

#[derive(Debug)]
struct ActiveProcess {
    metadata: MinecraftProcessMetadata,
    child: Child,
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
        child: Child,
    ) -> Result<(), ProcessError> {
        self.processes
            .insert(metadata.uuid(), ActiveProcess { metadata, child });

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
            return Ok(Some(
                process
                    .child
                    .try_wait()
                    .map_err(|_| ProcessError::WaitError { id: id.to_string() })?,
            ));
        }
        Ok(None)
    }

    async fn wait_for(&self, id: Uuid) -> Result<(), ProcessError> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process
                .child
                .wait()
                .await
                .map_err(|_| ProcessError::WaitError { id: id.to_string() })?;
        }
        Ok(())
    }

    async fn kill(&self, id: Uuid) -> Result<(), ProcessError> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process
                .child
                .kill()
                .await
                .map_err(|_| ProcessError::KillError { id: id.to_string() })?;
        }
        Ok(())
    }
}
