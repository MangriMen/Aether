mod app;
mod domain;
pub mod infra;

#[cfg(test)]
mod tests;

// ── Domain models (pure data containers) ──
pub use domain::{
    AtomicInstallParams, BaseContentParams, CapabilityMetadata, Checksum, ContentFile,
    ContentFileUpdateInfo, ContentInstallParams, ContentItem, ContentProviderCapabilityMetadata,
    ContentSearchParams, ContentSearchResult, ContentSourceCapabilityMetadata, ContentType,
    ContentVersion, ContentVersionDependency, ContentVersionDependencyType, ContentVersionStatus,
    ContentVersionType, CreateContentFileParams, DownloadInstruction, DownloadedContent,
    ImporterCapabilityMetadata, Instance, InstanceBuilder, InstanceError, InstanceField,
    InstanceInstallStage, InstanceSnapshot, InstanceValidationErrorReason, ModpackInstallParams,
    ModpackPayload, Pack, PackEntry, PackFile, PackFileDownload, PackFileOption, PackInfo,
    ProviderId, RequestedContentVersionStatus, UpdaterCapabilityMetadata, VersionInfo,
    VersionPayload,
};

// ── App layer: Use Cases (Inbound Ports) ──
pub use app::{
    ChangeContentState, ChangeContentStateUseCase, ChangeContentStateUseCasePort,
    CheckContentCompatibilityUseCase, CheckContentCompatibilityUseCasePort,
    ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentFileService,
    ContentGetParams, ContentListVersionsParams, ContentManagementPort, ContentProvider,
    ContentProviderPort, ContentSource, ContentStateAction, CreateInstanceUseCase,
    CreateInstanceUseCasePort, EditInstance, EditInstanceIcon, EditInstanceIconUseCase,
    EditInstanceIconUseCasePort, EditInstanceUseCase, EditInstanceUseCasePort, GetContentUseCase,
    GetContentUseCasePort, GetInstanceUseCase, GetInstanceUseCasePort, ImportContent,
    ImportContentUseCase, ImportContentUseCasePort, ImportInstance, ImportInstanceUseCase,
    ImportInstanceUseCasePort, Importer, InstallContentUseCase, InstallContentUseCasePort,
    InstallContentV2UseCasePort, InstallInstanceUseCase, InstanceCrudPort, InstanceFeature, InstanceFileService,
    InstanceInstallService, InstanceLaunchService, InstanceLifecyclePort, InstanceServicesPort,
    InstanceStorage, InstanceStorageExt, InstanceWatcherService, LaunchInstanceUseCase,
    LaunchInstanceWithActiveAccountUseCase, LaunchInstanceWithActiveAccountUseCasePort,
    LegacyInstallContentUseCase, ListContentUseCase, ListContentUseCasePort,
    ListContentVersionsUseCase, ListContentVersionsUseCasePort, ListImportersUseCase,
    ListImportersUseCasePort, ListInstancesUseCase, ListInstancesUseCasePort, ListProvidersUseCase,
    ListProvidersUseCasePort, NewInstance, PackLifecycleHandler, PackLifecycleHandlerRegistry,
    PackStorage, RemoveContent, RemoveContentUseCase, RemoveContentUseCasePort,
    RemoveInstanceUseCase, RemoveInstanceUseCasePort, SearchContentUseCase,
    SearchContentUseCasePort, UpdateInstanceUseCase, UpdateInstanceUseCasePort, Updater,
};
