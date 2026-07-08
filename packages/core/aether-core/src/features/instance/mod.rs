mod app;
mod domain;
pub mod infra;

#[cfg(test)]
mod tests;

// ── Domain models (pure data containers) ──
pub use domain::{
    AtomicInstallParams, BaseContentParams, CapabilityMetadata, ContentFile, ContentFileUpdateInfo,
    ContentInstallParams, ContentItem, ContentProviderCapabilityMetadata, ContentSearchParams,
    ContentSearchResult, ContentType, ContentVersion, ContentVersionDependency,
    ContentVersionDependencyType, ContentVersionStatus, ContentVersionType,
    CreateContentFileParams, DownloadedContent, ImporterCapabilityMetadata, Instance,
    InstanceBuilder, InstanceError, InstanceField, InstanceInstallStage, InstanceSnapshot,
    InstanceValidationErrorReason, ModpackInstallParams, Pack, PackEntry, PackFile,
    PackFileDownload, PackFileOption, PackInfo, ProviderId, RequestedContentVersionStatus,
    UpdaterCapabilityMetadata,
};

// ── App layer: Use Cases (Inbound Ports) ──
pub use app::{
    ChangeContentState, ChangeContentStateUseCase, ChangeContentStateUseCasePort,
    CheckContentCompatibilityUseCase, CheckContentCompatibilityUseCasePort,
    ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentFileService,
    ContentGetParams, ContentListVersionsParams, ContentProvider, ContentStateAction,
    CreateInstanceUseCase, CreateInstanceUseCasePort, EditInstance, EditInstanceIcon,
    EditInstanceIconUseCase, EditInstanceIconUseCasePort, EditInstanceUseCase,
    EditInstanceUseCasePort, GetContentUseCase, GetContentUseCasePort, GetInstanceUseCase,
    GetInstanceUseCasePort, ImportContent, ImportContentUseCase, ImportContentUseCasePort,
    ImportInstance, ImportInstanceUseCase, ImportInstanceUseCasePort, Importer,
    InstallContentUseCase, InstallContentUseCasePort, InstallInstanceUseCase, InstanceFeature,
    InstanceFileService, InstanceInstallService, InstanceLaunchService, InstanceStorage,
    InstanceStorageExt, InstanceWatcherService, LaunchInstanceUseCase,
    LaunchInstanceWithActiveAccountUseCase, LaunchInstanceWithActiveAccountUseCasePort,
    ListContentUseCase, ListContentUseCasePort, ListContentVersionsUseCase,
    ListContentVersionsUseCasePort, ListImportersUseCase, ListImportersUseCasePort,
    ListInstancesUseCase, ListInstancesUseCasePort, ListProvidersUseCase, ListProvidersUseCasePort,
    NewInstance, PackStorage, RemoveContent, RemoveContentUseCase, RemoveContentUseCasePort,
    RemoveInstanceUseCase, RemoveInstanceUseCasePort, SearchContentUseCase,
    SearchContentUseCasePort, UpdateInstanceUseCase, UpdateInstanceUseCasePort, Updater,
};
