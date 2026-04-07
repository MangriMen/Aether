use async_trait::async_trait;

use crate::features::instance::{Instance, InstanceError};

#[async_trait]
pub trait InstanceStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError>;
    async fn get(&self, id: &str) -> Result<Instance, InstanceError>;
    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError>;
    async fn remove(&self, id: &str) -> Result<(), InstanceError>;
}
