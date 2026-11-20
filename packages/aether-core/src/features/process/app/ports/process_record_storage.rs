use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::features::instance::InstanceId;
use crate::features::process::domain::ProcessRecord;
use crate::shared::error::DatabaseError;

/// The started games kept across launcher restarts. Removing or touching a record that is not
/// there is not an error: a game whose record could not be saved still runs to its end.
#[async_trait]
pub trait ProcessRecordStorage: Send + Sync {
    async fn insert(&self, record: &ProcessRecord) -> Result<(), DatabaseError>;
    async fn remove(&self, id: Uuid) -> Result<(), DatabaseError>;
    /// Every readable record; one that cannot be read is left out.
    async fn list(&self) -> Result<Vec<ProcessRecord>, DatabaseError>;
    async fn list_for_instance(
        &self,
        instance_id: &InstanceId,
    ) -> Result<Vec<ProcessRecord>, DatabaseError>;
    /// Moves the playtime mark of the run to `until`.
    async fn touch_playtime(&self, id: Uuid, until: DateTime<Utc>) -> Result<(), DatabaseError>;
}
