mod app;
mod domain;
pub mod infra;

#[cfg(test)]
mod tests;

// ── Domain models (pure data containers) ──
pub use domain::{
    AnchoredPath, AtomicInstallParams, BaseContentParams, BoxedSource, BrokenInstance,
    BrokenInstanceOrigin, BrokenInstanceReason, BrokenRecord, CapabilityId, CapabilityMetadata,
    ChannelVersion, ContentAuthor, ContentDownloadFailure, ContentFile, ContentFileUpdateInfo,
    ContentFileView, ContentId, ContentIdentity, ContentInstallParams, ContentItem,
    ContentMetadata, ContentPath, ContentProviderCapabilityMetadata, ContentProviderRejection,
    ContentSearchParams, ContentSearchResult, ContentSource, ContentType, ContentVersion,
    ContentVersionDependency, ContentVersionDependencyType, ContentVersionFile, ContentVersionId,
    ContentVersionMetadata, ContentVersionStatus, ContentVersionType, FileSide,
    ImporterCapabilityMetadata, InstallFileEntry, InstallPlan, InstallTarget, Instance,
    InstanceBuilder, InstanceError, InstanceField, InstanceGame, InstanceId, InstanceInstallStage,
    InstanceName, InstanceRelativePath, InstanceSnapshot, InstanceValidationErrorReason,
    InvalidContentId, InvalidId, InvalidPagination, InvalidRelativePath, ModpackInstallParams,
    NewInstance, OptionalFile, OverrideLocation, OverrideSource, Pack, PackEntry, PackFile,
    PackFileDownload, PackFileOption, PackInfo, PackMetadata, PackSource, PageIndex, PageSize,
    PaginationField, PaginationRejection, PlanOrigin, PlatformId, PluginId, ProviderId,
    ProviderIdError, ReadFailure, ReleaseChannel, RequestedContentVersionStatus,
    ResolveInstallPlanParams, ResolveTarget, ResolvedFile, StorageError, UpdaterCapabilityMetadata,
    folder_key, is_valid_id, parse_file_hash, select_version,
};

// ── App layer: Use Cases (Inbound Ports) ──
pub use app::{
    BrokenInstanceSource, CachedContentAuthor, CachedContentIcon, CachedContentMetadata,
    CachedContentVersionMetadata, ChangeContentState, ChangeContentStateUseCase,
    ChangeContentStateUseCasePort, ChangeInstanceGameUseCase, ChangeInstanceGameUseCasePort,
    CheckContentCompatibilityUseCase, CheckContentCompatibilityUseCasePort,
    ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentDownloader,
    ContentFileService, ContentGetParams, ContentGetVersionParams, ContentIconCache,
    ContentIconError, ContentListVersionsParams, ContentManagementPort, ContentMetadataCache,
    ContentProvider, ContentProviderPort, ContentStateAction, CreateInstanceUseCase,
    CreateInstanceUseCasePort, DownloadEvent, DownloadReport, EditInstance, EditInstanceIcon,
    EditInstanceIconUseCase, EditInstanceIconUseCasePort, EditInstanceUseCase,
    EditInstanceUseCasePort, GetContentUseCase, GetContentUseCasePort, GetContentVersionUseCase,
    GetContentVersionUseCasePort, GetInstanceUseCase, GetInstanceUseCasePort, ImportContent,
    ImportContentUseCase, ImportContentUseCasePort, ImportInstance, ImportInstanceUseCase,
    ImportInstanceUseCasePort, Importer, InstallCancellations, InstallContentUseCase,
    InstallContentUseCasePort, InstallInstanceUseCase, InstallPlanExecutor,
    InstallPlanExecutorPort, InstanceChange, InstanceCrudPort, InstanceFeature,
    InstanceFileService, InstanceInstallService, InstanceLaunchService, InstanceLifecyclePort,
    InstanceLockGuard, InstanceLocks, InstanceServicesPort, InstanceStorage, InstanceStorageExt,
    InstanceWatcherService, LaunchInstanceUseCase, LaunchInstanceWithActiveAccountUseCase,
    LaunchInstanceWithActiveAccountUseCasePort, LegacyInstanceSource, ListBrokenInstancesUseCase,
    ListBrokenInstancesUseCasePort, ListContentUseCase, ListContentUseCasePort,
    ListContentVersionsUseCase, ListContentVersionsUseCasePort, ListImportersUseCase,
    ListImportersUseCasePort, ListInstancesUseCase, ListInstancesUseCasePort, ListProvidersUseCase,
    ListProvidersUseCasePort, LockRefused, PackStorage, PlatformProviderSelection,
    RemoveBrokenInstanceUseCase, RemoveBrokenInstanceUseCasePort, RemoveContent,
    RemoveContentUseCase, RemoveContentUseCasePort, RemoveInstanceUseCase,
    RemoveInstanceUseCasePort, SearchContentUseCase, SearchContentUseCasePort,
    UpdateInstanceUseCase, UpdateInstanceUseCasePort, Updater, remove_plan_leftovers,
    reset_stuck_install_stages,
};

// Used inside the crate only.
pub(crate) use app::ContentMetadataService;
#[cfg(test)]
pub(crate) use app::{CONTENT_AUTHOR_TTL, CONTENT_METADATA_TTL, CONTENT_VERSION_METADATA_TTL};
pub use app::{ContentIconHousekeeping, IconAssetGuard, IconAssetReaper};
