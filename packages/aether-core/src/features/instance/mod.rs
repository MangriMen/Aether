mod app;
mod domain;
pub mod infra;

#[cfg(test)]
mod tests;

// ── Domain models (pure data containers) ──
pub use domain::{
    AtomicInstallParams, BaseContentParams, CapabilityMetadata, ChannelVersion, ContentFile,
    ContentFileUpdateInfo, ContentIdentity, ContentInstallParams, ContentItem, ContentMetadata,
    ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult, ContentSource,
    ContentType, ContentVersion, ContentVersionDependency, ContentVersionDependencyType,
    ContentVersionMetadata, ContentVersionStatus, ContentVersionType, FileSide,
    ImporterCapabilityMetadata, InstallFileEntry, InstallPlan, InstallTarget, Instance,
    InstanceBuilder, InstanceError, InstanceField, InstanceGame, InstanceInstallStage,
    InstanceSnapshot, InstanceValidationErrorReason, ModpackInstallParams, NewInstance,
    OptionalFile, OverrideLocation, OverrideSource, Pack, PackEntry, PackFile, PackFileDownload,
    PackFileOption, PackInfo, PackMetadata, PackSource, PlanOrigin, ProviderId, ReleaseChannel,
    RequestedContentVersionStatus, ResolveInstallPlanParams, ResolveTarget, ResolvedFile,
    UpdaterCapabilityMetadata, is_valid_id, select_version,
};

// ── App layer: Use Cases (Inbound Ports) ──
pub use app::{
    CONTENT_ICON_MAX_UNUSED, CONTENT_METADATA_TTL, CONTENT_VERSION_METADATA_TTL, CachedContentIcon,
    CachedContentMetadata, CachedContentVersionMetadata, CachedMetadata, ChangeContentState,
    ChangeContentStateUseCase, ChangeContentStateUseCasePort, ChangeInstanceGameUseCase,
    ChangeInstanceGameUseCasePort, CheckContentCompatibilityUseCase,
    CheckContentCompatibilityUseCasePort, ContentCompatibilityCheckParams,
    ContentCompatibilityResult, ContentDownloader, ContentFileService, ContentGetParams,
    ContentIconCache, ContentIconError, ContentListVersionsParams, ContentManagementPort,
    ContentMetadataCache, ContentMetadataService, ContentProvider, ContentProviderPort,
    ContentStateAction, CreateInstanceUseCase, CreateInstanceUseCasePort, EditInstance,
    EditInstanceIcon, EditInstanceIconUseCase, EditInstanceIconUseCasePort, EditInstanceUseCase,
    EditInstanceUseCasePort, GetContentUseCase, GetContentUseCasePort, GetInstanceUseCase,
    GetInstanceUseCasePort, ImportContent, ImportContentUseCase, ImportContentUseCasePort,
    ImportInstance, ImportInstanceUseCase, ImportInstanceUseCasePort, Importer,
    InstallContentUseCase, InstallContentUseCasePort, InstallInstanceUseCase, InstallPlanExecutor,
    InstallPlanExecutorPort, InstanceCrudPort, InstanceFeature, InstanceFileService,
    InstanceInstallService, InstanceLaunchService, InstanceLifecyclePort, InstanceLockGuard,
    InstanceLocks, InstanceServicesPort, InstanceStorage, InstanceStorageExt,
    InstanceWatcherService, LaunchInstanceUseCase, LaunchInstanceWithActiveAccountUseCase,
    LaunchInstanceWithActiveAccountUseCasePort, ListContentUseCase, ListContentUseCasePort,
    ListContentVersionsUseCase, ListContentVersionsUseCasePort, ListImportersUseCase,
    ListImportersUseCasePort, ListInstancesUseCase, ListInstancesUseCasePort, ListProvidersUseCase,
    ListProvidersUseCasePort, PackStorage, PlatformProviderSelection, RemoveContent,
    RemoveContentUseCase, RemoveContentUseCasePort, RemoveInstanceUseCase,
    RemoveInstanceUseCasePort, SearchContentUseCase, SearchContentUseCasePort,
    UpdateInstanceUseCase, UpdateInstanceUseCasePort, Updater, remove_plan_leftovers,
};
