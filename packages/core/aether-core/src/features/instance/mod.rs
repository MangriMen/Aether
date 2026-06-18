mod app;
mod domain;
pub mod infra;

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
    ChangeContentState, ChangeContentStateUseCase, CheckContentCompatibilityUseCase,
    ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentGetParams,
    ContentListVersionsParams, ContentProvider, ContentStateAction, CreateInstanceUseCase,
    EditInstance, EditInstanceIcon, EditInstanceIconUseCase, EditInstanceUseCase,
    GetContentUseCase, GetInstanceUseCase, ImportContent, ImportContentUseCase, ImportInstance,
    ImportInstanceUseCase, Importer, InstallContentUseCase, InstallInstanceUseCase,
    InstanceStorage, InstanceStorageExt, InstanceWatcherService, LaunchInstanceUseCase,
    LaunchInstanceWithActiveAccountUseCase, ListContentUseCase, ListContentVersionsUseCase,
    ListImportersUseCase, ListInstancesUseCase, ListProvidersUseCase, NewInstance, PackStorage,
    RemoveContent, RemoveContentUseCase, RemoveInstanceUseCase, SearchContentUseCase,
    UpdateInstanceUseCase, Updater,
};
