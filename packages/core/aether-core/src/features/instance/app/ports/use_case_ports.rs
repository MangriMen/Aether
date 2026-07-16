use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use dashmap::DashMap;

use crate::features::instance::app::{
    ChangeContentState, ContentCompatibilityCheckParams, ContentCompatibilityResult,
    ContentGetParams, ContentListVersionsParams, EditInstance, EditInstanceIcon, ImportContent,
    InstallPackRequest, NewInstance, RemoveContent,
};
use crate::features::instance::domain::{
    AtomicInstallParams, ContentFile, ContentItem, ContentProviderCapabilityMetadata,
    ContentVersion, Instance, InstanceError, PackManagerCapabilityMetadata,
};
use crate::features::instance::{ContentSearchParams, ContentSearchResult};
use crate::features::process::MinecraftProcessMetadata;
use crate::shared::capability::domain::CapabilityEntry;

#[async_trait]
pub trait CreateInstanceUseCasePort: Send + Sync {
    async fn execute(&self, new_instance: NewInstance) -> Result<String, InstanceError>;
}

#[async_trait]
pub trait GetInstanceUseCasePort: Send + Sync {
    async fn execute(&self, instance_id: String) -> Result<Instance, InstanceError>;
}

#[async_trait]
pub trait ListInstancesUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<Instance>, InstanceError>;
}

#[async_trait]
pub trait EditInstanceUseCasePort: Send + Sync {
    async fn execute(
        &self,
        instance_id: String,
        edit_instance: EditInstance,
    ) -> Result<Instance, InstanceError>;
}

#[async_trait]
pub trait EditInstanceIconUseCasePort: Send + Sync {
    async fn execute(
        &self,
        edit_instance_icon: EditInstanceIcon,
    ) -> Result<Instance, InstanceError>;
}

#[async_trait]
pub trait RemoveInstanceUseCasePort: Send + Sync {
    async fn execute(&self, instance_id: String) -> Result<(), InstanceError>;
}

#[async_trait]
pub trait UpdateInstanceUseCasePort: Send + Sync {
    async fn execute(&self, instance_id: String) -> Result<(), InstanceError>;
}

// ── Pack management ──

#[async_trait]
pub trait InstallPackUseCasePort: Send + Sync {
    async fn execute(&self, request: InstallPackRequest) -> Result<(), InstanceError>;
}

// ── Instance lifecycle ──

#[async_trait]
pub trait LaunchInstanceWithActiveAccountUseCasePort: Send + Sync {
    async fn execute(&self, instance_id: String)
    -> Result<MinecraftProcessMetadata, InstanceError>;
}

// ── Content management ──

#[async_trait]
pub trait ChangeContentStateUseCasePort: Send + Sync {
    async fn execute(&self, input: ChangeContentState) -> Result<(), InstanceError>;
}

#[async_trait]
pub trait ImportContentUseCasePort: Send + Sync {
    async fn execute(&self, input: ImportContent) -> Result<(), InstanceError>;
}

#[async_trait]
pub trait ListContentUseCasePort: Send + Sync {
    async fn execute(
        &self,
        instance_id: String,
    ) -> Result<DashMap<String, ContentFile>, InstanceError>;
}

#[async_trait]
pub trait RemoveContentUseCasePort: Send + Sync {
    async fn execute(&self, input: RemoveContent) -> Result<(), InstanceError>;
}

// ── Content providers ──

#[async_trait]
pub trait SearchContentUseCasePort: Send + Sync {
    async fn execute(
        &self,
        search_params: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError>;
}

#[async_trait]
pub trait GetContentUseCasePort: Send + Sync {
    async fn execute(&self, get_params: ContentGetParams) -> Result<ContentItem, InstanceError>;
}

#[async_trait]
pub trait InstallContentUseCasePort: Send + Sync {
    async fn execute(&self, install_params: AtomicInstallParams) -> Result<(), InstanceError>;
}

#[async_trait]
pub trait CheckContentCompatibilityUseCasePort: Send + Sync {
    async fn execute(
        &self,
        instance_ids: HashSet<String>,
        check_params: ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError>;
}

#[async_trait]
pub trait ListContentVersionsUseCasePort: Send + Sync {
    async fn execute(
        &self,
        get_params: ContentListVersionsParams,
    ) -> Result<Vec<ContentVersion>, InstanceError>;
}

#[async_trait]
pub trait ListProvidersUseCasePort: Send + Sync {
    async fn execute(
        &self,
    ) -> Result<Vec<CapabilityEntry<ContentProviderCapabilityMetadata>>, InstanceError>;
}

#[async_trait]
pub trait ListPackManagersUseCasePort: Send + Sync {
    async fn execute(
        &self,
    ) -> Result<Vec<CapabilityEntry<PackManagerCapabilityMetadata>>, InstanceError>;
}
