mod app;
mod domain;
pub mod infra;

#[cfg(test)]
mod tests;

// ── Domain models (pure data containers) ──
pub use domain::{
    AtomicInstallParams, BaseContentParams, CapabilityMetadata, ContentFile, ContentFileUpdateInfo,
    ContentItem, ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
    ContentType, ContentVersion, ContentVersionDependency, ContentVersionDependencyType,
    ContentVersionStatus, ContentVersionType, CreateContentFileParams, DownloadContext,
    DownloadedContent, Instance, InstanceBuilder, InstanceError, InstanceField,
    InstanceInstallStage, InstanceSnapshot, InstanceValidationErrorReason, Pack, PackEntry,
    PackFile, PackFileDownload, PackFileOption, PackInfo, PackInstallParams,
    PackManagerCapabilityMetadata, PackMetadata, PackSource, ProviderId,
    RequestedContentVersionStatus, UpdateStatus,
};

// ── App layer: Use Cases (Inbound Ports) ──
pub use app::{
    ChangeContentState, ChangeContentStateUseCase, ChangeContentStateUseCasePort,
    CheckContentCompatibilityUseCase, CheckContentCompatibilityUseCasePort,
    ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentFileService,
    ContentGetParams, ContentListVersionsParams, ContentManagementPort, ContentProvider,
    ContentProviderPort, ContentStateAction, CreateInstanceUseCase, CreateInstanceUseCasePort,
    EditInstance, EditInstanceIcon, EditInstanceIconUseCase, EditInstanceIconUseCasePort,
    EditInstanceUseCase, EditInstanceUseCasePort, GetContentUseCase, GetContentUseCasePort,
    GetInstanceUseCase, GetInstanceUseCasePort, ImportContent, ImportContentUseCase,
    ImportContentUseCasePort, InstallContentUseCase, InstallContentUseCasePort,
    InstallInstanceUseCase, InstallPackRequest, InstallPackUseCase, InstallPackUseCasePort,
    InstanceCrudPort, InstanceFeature, InstanceFileService, InstanceInstallService,
    InstanceLaunchService, InstanceLifecyclePort, InstanceServicesPort, InstanceStorage,
    InstanceStorageExt, InstanceWatcherService, LaunchInstanceUseCase,
    LaunchInstanceWithActiveAccountUseCase, LaunchInstanceWithActiveAccountUseCasePort,
    ListContentUseCase, ListContentUseCasePort, ListContentVersionsUseCase,
    ListContentVersionsUseCasePort, ListInstancesUseCase, ListInstancesUseCasePort,
    ListPackManagersUseCase, ListPackManagersUseCasePort, ListProvidersUseCase,
    ListProvidersUseCasePort, NewInstance, PackManager, PackStorage, RemoveContent,
    RemoveContentUseCase, RemoveContentUseCasePort, RemoveInstanceUseCase,
    RemoveInstanceUseCasePort, SearchContentUseCase, SearchContentUseCasePort,
    UpdateInstanceUseCase, UpdateInstanceUseCasePort,
};
