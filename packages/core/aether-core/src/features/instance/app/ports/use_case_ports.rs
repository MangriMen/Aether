use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use dashmap::DashMap;

use crate::features::instance::app::{
    ChangeContentState, ContentCompatibilityCheckParams, ContentCompatibilityResult,
    ContentGetParams, ContentListVersionsParams, EditInstance, EditInstanceIcon, ImportContent,
    ImportInstance, RemoveContent,
};
use crate::features::instance::domain::{
    ContentFile, ContentItem, ContentProviderCapabilityMetadata, ContentVersion,
    ImporterCapabilityMetadata, InstallPlan, Instance, InstanceError, InstanceGame, NewInstance,
    PlanOrigin,
};
use crate::features::instance::{ContentInstallParams, ContentSearchParams, ContentSearchResult};
use crate::features::process::MinecraftProcessMetadata;
use crate::shared::capability::domain::CapabilityEntry;

#[async_trait]
pub trait CreateInstanceUseCasePort: Send + Sync {
    async fn execute(&self, new_instance: NewInstance) -> Result<String, InstanceError>;

    /// Create an instance whose pack the caller is about to deploy: it stays `PackInstalling`
    /// (not launchable) once the game is installed, and the caller moves it to `Installed` when
    /// the pack's content is in.
    async fn execute_for_pack(&self, new_instance: NewInstance) -> Result<String, InstanceError>;
}

/// Moves an existing instance to another game. Nothing happens when it already runs it.
#[async_trait]
pub trait ChangeInstanceGameUseCasePort: Send + Sync {
    async fn execute(&self, instance_id: String, game: InstanceGame) -> Result<(), InstanceError>;
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

// ── Instance lifecycle ──

#[async_trait]
pub trait ImportInstanceUseCasePort: Send + Sync {
    async fn execute(&self, import_instance: ImportInstance) -> Result<(), InstanceError>;
}

#[async_trait]
pub trait ListImportersUseCasePort: Send + Sync {
    async fn execute(
        &self,
    ) -> Result<Vec<CapabilityEntry<ImporterCapabilityMetadata>>, InstanceError>;
}

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
    async fn execute(&self, install_params: ContentInstallParams) -> Result<(), InstanceError>;
}

/// Carries out an [`InstallPlan`]: the one path for installing
/// and updating content, a single mod and a whole modpack alike.
#[async_trait]
pub trait InstallPlanExecutorPort: Send + Sync {
    /// Apply `plan`, which `origin.provider_id` resolved for `origin.content_id`, and return
    /// the id of the instance it went into — the one it names, or the one it created.
    ///
    /// `origin.provider_id` is whose word the plan's direct URLs are: they are downloaded under
    /// its network policy (see [`ContentDownloader`](crate::features::instance::ContentDownloader)).
    /// The whole `origin` is what the installed files are recorded under: in an existing
    /// instance only the files of the same origin are replaced or removed (see [`PlanOrigin`]).
    async fn execute(&self, plan: InstallPlan, origin: PlanOrigin)
    -> Result<String, InstanceError>;
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
