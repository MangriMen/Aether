//! Maps the core's errors onto the typed errors a plugin receives from a host function.
//!
//! An error that wraps another feature's error (`InstanceError::MinecraftError`, a download
//! failure inside Java) is handed over as that feature's [`HostError`], so a plugin matches one
//! variant per cause instead of reading it out of text.

use aether_core_plugin_api::v0::{
    AuthErrorDto, EventErrorDto, FileWatcherErrorDto, HostError, InstanceErrorDto,
    InstanceFieldDto, InstanceValidationErrorReasonDto, JavaErrorDto, LoadConfigTypeDto,
    ManifestErrorDto, MinecraftErrorDto, PluginCallErrorDto, PluginErrorDto, PluginSourceTypeDto,
    ProcessErrorDto, RequestErrorDto, SettingsErrorDto,
};

use crate::{
    ErrorKind,
    features::{
        auth::{AuthApplicationError, AuthDomainError},
        events::EventError,
        file_watcher::FileWatcherError,
        instance::{InstanceError, InstanceField, InstanceValidationErrorReason},
        java::{JavaApplicationError, JavaDomainError},
        minecraft::{MinecraftApplicationError, MinecraftDomainError},
        plugins::{
            LoadConfigType, ManifestError, PluginCallError, PluginError, PluginProviderError,
            PluginSourceType,
        },
        process::ProcessError,
        settings::SettingsError,
    },
    shared::request_client::RequestError,
};

impl From<&ErrorKind> for HostError {
    fn from(value: &ErrorKind) -> Self {
        match value {
            ErrorKind::AuthError(err) => err.into(),
            ErrorKind::EventError(err) => err.into(),
            ErrorKind::FileWatcherError(err) => err.into(),
            ErrorKind::InstanceError(err) => err.into(),
            ErrorKind::JavaError(err) => err.into(),
            ErrorKind::MinecraftError(err) => err.into(),
            ErrorKind::PluginError(err) => err.into(),
            ErrorKind::ProcessError(err) => err.into(),
            ErrorKind::SettingsError(err) => err.into(),
            ErrorKind::RequestError(err) => err.into(),
            // The inner message: `ErrorKind::CoreError`'s `Display` adds a "Core error: "
            // prefix, and the variant already says as much.
            ErrorKind::CoreError(message) => Self::CoreError(message.clone()),
        }
    }
}

// --- auth ---

impl From<&AuthApplicationError> for HostError {
    fn from(value: &AuthApplicationError) -> Self {
        Self::AuthError(match value {
            AuthApplicationError::Domain(err) => err.into(),
            AuthApplicationError::Storage(details) => AuthErrorDto::Storage(details.clone()),
        })
    }
}

impl From<&AuthDomainError> for AuthErrorDto {
    fn from(value: &AuthDomainError) -> Self {
        match value {
            AuthDomainError::CredentialsNotFound { id } => {
                Self::CredentialsNotFound { id: id.to_string() }
            }
            AuthDomainError::NoActiveCredentials => Self::NoActiveCredentials,
            AuthDomainError::InvalidUsernameLength { min, max } => Self::InvalidUsernameLength {
                min: *min,
                max: *max,
            },
            AuthDomainError::InvalidUsernameChars => Self::InvalidUsernameChars,
            AuthDomainError::InvalidAccountType => Self::InvalidAccountType,
            AuthDomainError::TokenExpired => Self::TokenExpired,
        }
    }
}

// --- events ---

impl From<&EventError> for HostError {
    fn from(value: &EventError) -> Self {
        Self::EventError(match value {
            EventError::NotInitialized => EventErrorDto::NotInitialized,
            EventError::NoLoadingBar(id) => EventErrorDto::NoLoadingBar { id: id.to_string() },
            EventError::SerializeError(err) => EventErrorDto::SerializeError(err.to_string()),
            EventError::StorageError(err) => EventErrorDto::StorageError(err.to_string()),
        })
    }
}

// --- file watcher ---

impl From<&FileWatcherError> for HostError {
    fn from(value: &FileWatcherError) -> Self {
        Self::FileWatcherError(match value {
            FileWatcherError::PathNotFound { path } => FileWatcherErrorDto::PathNotFound {
                path: path.to_string_lossy().into_owned(),
            },
            FileWatcherError::WatchNotFound => FileWatcherErrorDto::WatchNotFound,
            FileWatcherError::NotifyError(err) => FileWatcherErrorDto::NotifyError(err.clone()),
        })
    }
}

// --- instance ---

impl From<&InstanceError> for HostError {
    fn from(value: &InstanceError) -> Self {
        match value {
            InstanceError::SettingsLoadError(err) => err.into(),
            InstanceError::MinecraftError(err) => err.into(),
            InstanceError::ProcessError(err) => err.into(),
            InstanceError::CredentialsError(err) => err.into(),
            other => Self::InstanceError(instance_dto(other)),
        }
    }
}

// One flat arm per variant; splitting an exhaustive mapping only hides which ones are covered.
#[allow(clippy::too_many_lines)]
fn instance_dto(value: &InstanceError) -> InstanceErrorDto {
    use InstanceErrorDto as Dto;

    match value {
        InstanceError::Storage(err) => Dto::Storage(err.clone()),
        InstanceError::NotFound { instance_id } => Dto::NotFound {
            instance_id: instance_id.clone(),
        },
        InstanceError::ContentProviderNotFound {
            plugin_id,
            capability_id,
        } => Dto::ContentProviderNotFound {
            plugin_id: plugin_id.clone(),
            capability_id: capability_id.clone(),
        },
        InstanceError::CapabilityOperationError => Dto::CapabilityOperationError,
        InstanceError::InstanceStillInstalling { instance_id } => Dto::InstanceStillInstalling {
            instance_id: instance_id.clone(),
        },
        InstanceError::InstanceAlreadyRunning {
            instance_id,
            process_id,
        } => Dto::InstanceAlreadyRunning {
            instance_id: instance_id.clone(),
            process_id: process_id.to_string(),
        },
        InstanceError::PrelaunchCommandError { code } => Dto::PrelaunchCommandError { code: *code },
        InstanceError::ValidationError { field, reason } => Dto::ValidationError {
            field: match field {
                InstanceField::Name => InstanceFieldDto::Name,
            },
            reason: match reason {
                InstanceValidationErrorReason::CannotBeEmpty => {
                    InstanceValidationErrorReasonDto::CannotBeEmpty
                }
            },
        },
        InstanceError::ImporterNotFound { importer_id } => Dto::ImporterNotFound {
            importer_id: importer_id.clone(),
        },
        InstanceError::ImportFailed {
            plugin_id,
            capability_id,
            reason,
        } => Dto::ImportFailed {
            plugin_id: plugin_id.clone(),
            capability_id: capability_id.clone(),
            reason: reason.clone(),
        },
        // No kind of its own, so as not to break decoding in plugins built without it.
        InstanceError::PluginNotRunning { .. } => Dto::ContentProviderError {
            reason: value.to_string(),
        },
        InstanceError::PackInfoNotFound => Dto::PackInfoNotFound,
        InstanceError::UpdaterNotFound { modpack_id } => Dto::UpdaterNotFound {
            modpack_id: modpack_id.clone(),
        },
        InstanceError::UpdateFailed { modpack_id } => Dto::UpdateFailed {
            modpack_id: modpack_id.clone(),
        },
        InstanceError::UnmanagedInstance { instance_id } => Dto::UnmanagedInstance {
            instance_id: instance_id.clone(),
        },
        InstanceError::ContentDuplication { content_path } => Dto::ContentDuplication {
            content_path: content_path.clone(),
        },
        InstanceError::ContentFilename { path } => Dto::ContentFilename {
            path: path.to_string_lossy().into_owned(),
        },
        InstanceError::InvalidRelativePath { path, reason } => Dto::InvalidRelativePath {
            path: path.clone(),
            reason: reason.clone(),
        },
        InstanceError::PathEscapesInstance { path, instance_id } => Dto::PathEscapesInstance {
            path: path.clone(),
            instance_id: instance_id.clone(),
        },
        InstanceError::FileTooLarge { path, size, limit } => Dto::FileTooLarge {
            path: path.clone(),
            size: *size,
            limit: *limit,
        },
        InstanceError::ContentDownloadError(err) => Dto::ContentDownloadError(err.clone()),
        InstanceError::ContentProviderError { reason } => Dto::ContentProviderError {
            reason: reason.clone(),
        },
        InstanceError::ContentForGameVersionNotFound { game_version } => {
            Dto::ContentForGameVersionNotFound {
                game_version: game_version.clone(),
            }
        }
        InstanceError::UnsupportedContentType { content_type } => Dto::UnsupportedContentType {
            content_type: (*content_type).into(),
        },
        InstanceError::UnsupportedOperation {
            provider_id,
            operation,
        } => Dto::UnsupportedOperation {
            provider_id: provider_id.clone().into(),
            operation: (*operation).to_owned(),
        },
        InstanceError::PlatformProviderNotFound { platform } => Dto::PlatformProviderNotFound {
            platform: platform.clone(),
        },
        InstanceError::PlatformProviderNotChosen { platform } => Dto::PlatformProviderNotChosen {
            platform: platform.clone(),
        },
        InstanceError::GameChangeNotAllowed {
            instance_id,
            provider_id,
            content_id,
        } => Dto::GameChangeNotAllowed {
            instance_id: instance_id.clone(),
            provider_id: provider_id.clone().into(),
            content_id: content_id.clone(),
        },
        // Errors of other features are handed over by `From<&InstanceError> for HostError`.
        InstanceError::SettingsLoadError(_)
        | InstanceError::MinecraftError(_)
        | InstanceError::ProcessError(_)
        | InstanceError::CredentialsError(_) => {
            unreachable!("wrapped errors are mapped by their own feature")
        }
    }
}

// --- java ---

impl From<&JavaApplicationError> for HostError {
    fn from(value: &JavaApplicationError) -> Self {
        match value {
            JavaApplicationError::Domain(err) => Self::JavaError(err.into()),
            JavaApplicationError::DownloadFailed(err) => err.into(),
        }
    }
}

impl From<&JavaDomainError> for JavaErrorDto {
    fn from(value: &JavaDomainError) -> Self {
        match value {
            JavaDomainError::NotFound { version } => Self::NotFound { version: *version },
            JavaDomainError::InvalidPath { path } => Self::InvalidPath {
                path: path.to_string_lossy().into_owned(),
            },
            JavaDomainError::EmptyPath => Self::EmptyPath,
            JavaDomainError::InvalidVersion { version } => Self::InvalidVersion {
                version: version.clone(),
            },
            JavaDomainError::FailedToGetProperties { reason } => Self::FailedToGetProperties {
                reason: reason.clone(),
            },
            JavaDomainError::VersionNotAvailable { version, os, arch } => {
                Self::VersionNotAvailable {
                    version: *version,
                    os: os.clone(),
                    arch: arch.clone(),
                }
            }
            JavaDomainError::VersionGetFailed { version, os, arch } => Self::VersionGetFailed {
                version: *version,
                os: os.clone(),
                arch: arch.clone(),
            },
            JavaDomainError::FailedToInstall { version, os, arch } => Self::FailedToInstall {
                version: *version,
                os: os.clone(),
                arch: arch.clone(),
            },
            JavaDomainError::RemoveOldInstallationError { path } => {
                Self::RemoveOldInstallationError {
                    path: path.to_string_lossy().into_owned(),
                }
            }
            JavaDomainError::Storage(err) => Self::Storage(err.clone()),
        }
    }
}

// --- minecraft ---

impl From<&MinecraftApplicationError> for HostError {
    fn from(value: &MinecraftApplicationError) -> Self {
        match value {
            MinecraftApplicationError::Domain(err) => Self::MinecraftError(err.into()),
            MinecraftApplicationError::JavaError(err) => err.into(),
            MinecraftApplicationError::Storage(err) => {
                Self::MinecraftError(MinecraftErrorDto::StorageFailure {
                    reason: err.to_string(),
                })
            }
        }
    }
}

impl From<&MinecraftDomainError> for MinecraftErrorDto {
    fn from(value: &MinecraftDomainError) -> Self {
        match value {
            MinecraftDomainError::VersionNotFound { version } => Self::VersionNotFound {
                version: version.clone(),
            },
            MinecraftDomainError::VersionForLoaderNotFound {
                loader_version_preference,
            } => Self::VersionForLoaderNotFound {
                loader_version_preference: loader_version_preference.clone().into(),
            },
            MinecraftDomainError::LoaderNotFound {
                loader_version_preference,
            } => Self::LoaderNotFound {
                loader_version_preference: loader_version_preference.clone().into(),
            },
            MinecraftDomainError::LoaderVersionRequired => Self::LoaderVersionRequired,
            MinecraftDomainError::DefaultLoaderNotFound => Self::DefaultLoaderNotFound,
            MinecraftDomainError::ProcessorFailed { reason } => Self::ProcessorFailed {
                reason: reason.clone(),
            },
            MinecraftDomainError::PreLaunchFailed { code } => Self::PreLaunchFailed { code: *code },
            MinecraftDomainError::PostLaunchFailed { code } => {
                Self::PostLaunchFailed { code: *code }
            }
            MinecraftDomainError::PathNotFound { path, entity_type } => Self::PathNotFound {
                path: path.to_string_lossy().into_owned(),
                entity_type: entity_type.clone(),
            },
            MinecraftDomainError::ParseFailed { reason } => Self::ParseFailed {
                reason: reason.clone(),
            },
            MinecraftDomainError::StorageFailure { reason } => Self::StorageFailure {
                reason: reason.clone(),
            },
        }
    }
}

// --- plugins ---

impl From<&PluginError> for HostError {
    fn from(value: &PluginError) -> Self {
        match value {
            PluginError::Settings(err) => err.into(),
            PluginError::Provider(err) => Self::PluginError(err.into()),
            other => Self::PluginError(plugin_dto(other)),
        }
    }
}

fn plugin_dto(value: &PluginError) -> PluginErrorDto {
    use PluginErrorDto as Dto;

    match value {
        PluginError::NotFound { plugin_id } => Dto::NotFound {
            plugin_id: plugin_id.clone(),
        },
        PluginError::AlreadyLoaded { plugin_id } => Dto::AlreadyLoaded {
            plugin_id: plugin_id.clone(),
        },
        PluginError::LoadingInProgress { plugin_id } => Dto::LoadingInProgress {
            plugin_id: plugin_id.clone(),
        },
        // `Busy` and `Stopped` reach a plugin only through a host function that reaches another
        // plugin. They map to the closest kinds the ABI has: a new kind would not decode in
        // plugins built before it.
        PluginError::UnloadingInProgress { plugin_id } | PluginError::Busy { plugin_id } => {
            Dto::UnloadingInProgress {
                plugin_id: plugin_id.clone(),
            }
        }
        PluginError::AlreadyUnloaded { plugin_id } | PluginError::Stopped { plugin_id } => {
            Dto::AlreadyUnloaded {
                plugin_id: plugin_id.clone(),
            }
        }
        PluginError::LoadFailed { plugin_id, reason } => Dto::LoadFailed {
            plugin_id: plugin_id.clone(),
            reason: reason.clone(),
        },
        PluginError::IncompatibleApiVersion { plugin_id, reason } => Dto::IncompatibleApiVersion {
            plugin_id: plugin_id.clone(),
            reason: reason.clone(),
        },
        PluginError::FunctionCallFailed {
            function_name,
            plugin_id,
            error,
        } => Dto::FunctionCallFailed {
            function_name: function_name.clone(),
            plugin_id: plugin_id.clone(),
            error: error.into(),
        },
        PluginError::Manifest(err) => Dto::Manifest(err.into()),
        PluginError::LoaderNotFound { config_type } => Dto::LoaderNotFound {
            config_type: match config_type {
                LoadConfigType::Extism => LoadConfigTypeDto::Extism,
            },
        },
        PluginError::CapabilityRegistrationFailed {
            capability_type,
            capability_id,
        } => Dto::CapabilityRegistrationFailed {
            capability_type: (*capability_type).to_owned(),
            capability_id: capability_id.clone(),
        },
        PluginError::CapabilityCancelRegistrationFailed {
            capability_type,
            capability_id,
        } => Dto::CapabilityCancelRegistrationFailed {
            capability_type: (*capability_type).to_owned(),
            capability_id: capability_id.clone(),
        },
        // Only the user's settings, the import dialog and an update produce the last ones; the ABI's closest kind,
        // as a new one would not decode in plugins built before it.
        PluginError::AccessViolation { plugin_id, path }
        | PluginError::IdMismatch {
            plugin_id,
            archive_id: path,
        }
        | PluginError::InvalidAllowedPath {
            plugin_id, path, ..
        }
        | PluginError::InvalidAllowedHost {
            plugin_id,
            url: path,
            ..
        } => Dto::AccessViolation {
            plugin_id: plugin_id.clone(),
            path: path.clone(),
        },
        PluginError::HashComputationFailed => Dto::HashComputationFailed,
        PluginError::Storage(err) => Dto::Storage(err.to_string()),
        PluginError::Settings(_) | PluginError::Provider(_) => {
            unreachable!("mapped by `From<&PluginError> for HostError`")
        }
    }
}

impl From<&PluginProviderError> for PluginErrorDto {
    fn from(value: &PluginProviderError) -> Self {
        match value {
            PluginProviderError::ManifestNotFound { path } => {
                Self::ManifestNotFound { path: path.clone() }
            }
            PluginProviderError::InvalidManifestFormat { error } => Self::InvalidManifestFormat {
                error: error.clone(),
            },
            PluginProviderError::ExtractionFailed { from } => {
                Self::ExtractionFailed { from: from.clone() }
            }
            PluginProviderError::InvalidExtractionFormat => Self::InvalidExtractionFormat,
            PluginProviderError::FileExtractionFailed { from } => {
                Self::FileExtractionFailed { from: from.clone() }
            }
            PluginProviderError::Manifest(err) => Self::Manifest(err.into()),
            PluginProviderError::Storage(err) => Self::Storage(err.to_string()),
            PluginProviderError::ProviderFetchError {
                source_type,
                details,
            } => Self::ProviderFetchError {
                source_type: source_type.into(),
                details: details.clone(),
            },
            PluginProviderError::ProviderNoAssets { source_type } => Self::ProviderNoAssets {
                source_type: source_type.into(),
            },
            PluginProviderError::NoRemoteSource { plugin_id } => Self::NoRemoteSource {
                plugin_id: plugin_id.clone(),
            },
            PluginProviderError::DownloadFailed { url, details } => Self::DownloadFailed {
                url: url.clone(),
                details: details.clone(),
            },
            PluginProviderError::ProviderRateLimited {
                source_type,
                retry_after,
            } => Self::ProviderRateLimited {
                source_type: source_type.into(),
                retry_after: *retry_after,
            },
        }
    }
}

impl From<&PluginSourceType> for PluginSourceTypeDto {
    fn from(value: &PluginSourceType) -> Self {
        match value {
            PluginSourceType::GitHub => Self::GitHub,
            PluginSourceType::Local => Self::Local,
        }
    }
}

impl From<&PluginCallError> for PluginCallErrorDto {
    fn from(value: &PluginCallError) -> Self {
        match value {
            PluginCallError::MissingFunction => Self::MissingFunction,
            PluginCallError::Timeout => Self::Timeout,
            PluginCallError::OutOfFuel => Self::OutOfFuel,
            PluginCallError::OutOfMemory => Self::OutOfMemory,
            PluginCallError::Trap(message) => Self::Trap(message.clone()),
            PluginCallError::NonZeroExit(code) => Self::NonZeroExit(*code),
            PluginCallError::Plugin(message) => Self::Plugin(message.clone()),
            PluginCallError::Codec(message) => Self::Codec(message.clone()),
            PluginCallError::Reentrant => Self::Reentrant,
        }
    }
}

impl From<&ManifestError> for ManifestErrorDto {
    fn from(value: &ManifestError) -> Self {
        match value {
            ManifestError::UnsupportedApi => Self::UnsupportedApi,
            ManifestError::InvalidSemver {
                field,
                value,
                error,
            } => Self::InvalidSemver {
                field: field.clone(),
                value: value.clone(),
                error: error.clone(),
            },
            ManifestError::InvalidPathMapping => Self::InvalidPathMapping,
            ManifestError::UnknownHostFunctions(names) => Self::UnknownHostFunctions(names.clone()),
            ManifestError::HostFunctionsNotDeclared => Self::HostFunctionsNotDeclared,
            ManifestError::InvalidId { field, value } => Self::InvalidId {
                field: field.clone(),
                value: value.clone(),
            },
            ManifestError::ReservedPluginId => Self::ReservedPluginId,
            ManifestError::PlatformWithoutResolveFile { capability_id } => {
                Self::PlatformWithoutResolveFile {
                    capability_id: capability_id.clone(),
                }
            }
        }
    }
}

// --- process ---

impl From<&ProcessError> for HostError {
    fn from(value: &ProcessError) -> Self {
        Self::ProcessError(match value {
            ProcessError::KillError { id } => ProcessErrorDto::KillError { id: id.clone() },
            ProcessError::WaitError { id } => ProcessErrorDto::WaitError { id: id.clone() },
            ProcessError::Io(err) => ProcessErrorDto::Io(err.to_string()),
        })
    }
}

// --- settings ---

impl From<&SettingsError> for HostError {
    fn from(value: &SettingsError) -> Self {
        Self::SettingsError(match value {
            SettingsError::NotFound => SettingsErrorDto::NotFound,
            SettingsError::Storage(err) => SettingsErrorDto::Storage(err.clone()),
            // The plugin API has no variant for it and a plugin cannot edit the provider choice,
            // so it only ever sees it as a settings failure with the readable message.
            SettingsError::InvalidPlatformProvider { .. } => {
                SettingsErrorDto::Storage(value.to_string())
            }
        })
    }
}

// --- request ---

impl From<&RequestError> for HostError {
    fn from(value: &RequestError) -> Self {
        Self::RequestError(match value {
            RequestError::Acquire(err) => RequestErrorDto::Acquire(err.to_string()),
            RequestError::RequestSend(err) => RequestErrorDto::RequestSend(err.to_string()),
            RequestError::Middleware(err) => RequestErrorDto::Middleware(err.to_string()),
            RequestError::Hash { actual, expected } => RequestErrorDto::Hash {
                actual: actual.clone(),
                expected: expected.clone(),
            },
            RequestError::Json(err) => RequestErrorDto::Json(err.to_string()),
            RequestError::Toml(err) => RequestErrorDto::Toml(err.to_string()),
            RequestError::Parse(err) => RequestErrorDto::Parse(err.to_string()),
            RequestError::Status { status, url } => RequestErrorDto::Status {
                status: *status,
                url: url.clone(),
            },
            RequestError::TooLarge { limit } => RequestErrorDto::TooLarge { limit: *limit },
            RequestError::Io { path, source } => RequestErrorDto::Io {
                path: path.display().to_string(),
                details: source.to_string(),
            },
        })
    }
}
