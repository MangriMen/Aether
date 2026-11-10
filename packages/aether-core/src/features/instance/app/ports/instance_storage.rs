use async_trait::async_trait;

use crate::features::instance::domain::{BrokenRecord, Instance, InstanceError, InstanceId};

/// A change made to an instance in place by [`InstanceStorage::update`]; returns whether it
/// changed anything.
pub type InstanceChange<'a> =
    Box<dyn FnOnce(&mut Instance) -> Result<bool, InstanceError> + Send + 'a>;

#[async_trait]
pub trait InstanceStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError>;
    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError>;
    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError>;
    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError>;

    /// The id of every stored record, those that cannot be read included. Without a storage that
    /// keeps unreadable records, that is the ids of the instances it lists.
    async fn stored_ids(&self) -> Result<Vec<String>, InstanceError> {
        Ok(self
            .list()
            .await?
            .iter()
            .map(|instance| instance.id().to_string())
            .collect())
    }

    /// Read-modify-write of one instance. `change` returns whether it changed anything;
    /// nothing is written otherwise. Implementations must make the whole cycle atomic with
    /// every other write, so concurrent edits do not overwrite each other's fields. Writing
    /// back a whole instance read earlier with [`upsert`](Self::upsert) loses such edits.
    async fn update(
        &self,
        id: &InstanceId,
        change: InstanceChange<'_>,
    ) -> Result<Instance, InstanceError>;
}

/// The records a storage holds but cannot read.
#[async_trait]
pub trait BrokenInstanceSource: Send + Sync {
    async fn list_broken(&self) -> Result<Vec<BrokenRecord>, InstanceError>;
}
