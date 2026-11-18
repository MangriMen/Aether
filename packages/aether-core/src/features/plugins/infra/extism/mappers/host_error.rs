//! Maps the core's errors onto the [`HostError`] a plugin receives from a host function.
//!
//! This is the one place that decides what kind an error is for a plugin: what it can do about
//! it, not which feature failed. Each enum is matched variant by variant, so a new variant has
//! to be placed here before it compiles. An error that wraps another feature's error is
//! classified by the wrapped one. The message is the core's own text of the error.

use std::{error::Error, fmt::Display};

use aether_core_plugin_api::v0::{CommandRefusalDto, HostError, HttpErrorDto};

use crate::{
    ErrorKind,
    features::{
        instance::{ContentDownloadFailure, ContentProviderRejection, InstanceError, StorageError},
        java::{JavaApplicationError, JavaDomainError},
        minecraft::{MinecraftApplicationError, MinecraftDomainError},
        plugins::{
            CommandApprovalRefusal, PluginCallError, PluginCommandError, PluginDownloadError,
            PluginError, PluginHttpError, PluginProviderError, ProviderFailure,
        },
        settings::SettingsError,
    },
    shared::{io::IoError, request_client::RequestError},
};

/// The kinds of [`HostError`] that carry nothing but the message.
#[derive(Debug, Clone, Copy)]
enum Kind {
    InvalidArgument,
    NotFound,
    AccessDenied,
    TooLarge,
    Conflict,
    Cancelled,
    TimedOut,
    PluginCallFailed,
    Internal,
}

impl Kind {
    fn of(self, error: &impl Display) -> HostError {
        let message = error.to_string();
        match self {
            Self::InvalidArgument => HostError::InvalidArgument { message },
            Self::NotFound => HostError::NotFound { message },
            Self::AccessDenied => HostError::AccessDenied { message },
            Self::TooLarge => HostError::TooLarge { message },
            Self::Conflict => HostError::Conflict { message },
            Self::Cancelled => HostError::Cancelled { message },
            Self::TimedOut => HostError::TimedOut { message },
            Self::PluginCallFailed => HostError::PluginCallFailed { message },
            Self::Internal => HostError::Internal { message },
        }
    }
}

impl From<&ErrorKind> for HostError {
    fn from(value: &ErrorKind) -> Self {
        match value {
            ErrorKind::InstanceError(err) => instance(err),
            ErrorKind::JavaError(err) => java(err),
            ErrorKind::MinecraftError(err) => minecraft(err),
            ErrorKind::PluginError(err) => plugin(err),
            ErrorKind::SettingsError(err) => settings(err),
            ErrorKind::RequestError(err) => request(err),
            ErrorKind::PluginHttpError(err) => http(err, plugin_http(err)),
            // The text names a host path, which stays in the log.
            ErrorKind::PluginDownloadError(PluginDownloadError::UnusableCache { .. }) => {
                Kind::Internal.of(&"The plugin cache cannot be used")
            }
            ErrorKind::PluginDownloadError(err) => http_or_internal(err, plugin_download(err)),
            ErrorKind::PluginCommandError(err) => plugin_command(err),
            // No host function reaches these: should one, the plugin can do nothing about it.
            ErrorKind::AuthError(err) => Kind::Internal.of(err),
            ErrorKind::EventError(err) => Kind::Internal.of(err),
            ErrorKind::FileWatcherError(err) => Kind::Internal.of(err),
            ErrorKind::GameLogError(err) => Kind::Internal.of(err),
            ErrorKind::ProcessError(err) => Kind::Internal.of(err),
            // The inner message: `ErrorKind::CoreError`'s `Display` adds a "Core error: " prefix.
            ErrorKind::CoreError(message) => Kind::Internal.of(message),
        }
    }
}

fn instance(value: &InstanceError) -> HostError {
    let kind = match value {
        InstanceError::ValidationError { .. }
        | InstanceError::InvalidRelativePath { .. }
        | InstanceError::InvalidContentId { .. }
        | InstanceError::InvalidGameVersion { .. }
        | InstanceError::InvalidPagination { .. }
        | InstanceError::InvalidLoaderVersion { .. }
        | InstanceError::ContentFilename { .. }
        | InstanceError::NotAContentFile { .. }
        | InstanceError::IconNotSupported
        | InstanceError::UnsupportedContentType { .. }
        | InstanceError::UnsupportedOperation { .. }
        | InstanceError::UnmanagedInstance { .. }
        | InstanceError::GameChangeNotAllowed { .. } => Kind::InvalidArgument,

        InstanceError::NotFound { .. }
        | InstanceError::ContentProviderNotFound { .. }
        | InstanceError::ImporterNotFound { .. }
        | InstanceError::UpdaterNotFound { .. }
        | InstanceError::PackInfoNotFound
        | InstanceError::PackFileNotFound { .. }
        | InstanceError::ContentForGameVersionNotFound { .. }
        | InstanceError::PlatformProviderNotFound { .. } => Kind::NotFound,

        InstanceError::PathEscapesInstance { .. } => Kind::AccessDenied,

        InstanceError::FileTooLarge { .. } | InstanceError::IconTooLarge { .. } => Kind::TooLarge,

        InstanceError::InstanceStillInstalling { .. }
        | InstanceError::InstanceAlreadyRunning { .. }
        | InstanceError::ContentDuplication { .. }
        | InstanceError::PluginNotRunning { .. }
        | InstanceError::CachesBeingCleared
        | InstanceError::NotBroken { .. }
        | InstanceError::PlatformProviderNotChosen { .. } => Kind::Conflict,

        InstanceError::Cancelled => Kind::Cancelled,

        InstanceError::PrelaunchCommandTimeout { .. } => Kind::TimedOut,

        // The host was refused to the plugin behind the capability, not to the one that asked
        // for it: as `http` / `hostNotAllowed` it would read as the caller's own refusal.
        InstanceError::ImportFailed { .. }
        | InstanceError::PluginHostNotAllowed { .. }
        | InstanceError::PluginAddressBlocked { .. } => Kind::PluginCallFailed,

        InstanceError::Storage(err) if storage_is_missing(err) => Kind::NotFound,

        InstanceError::Storage(_)
        | InstanceError::InvalidRecord { .. }
        | InstanceError::ContentStateChangeFailed { .. }
        | InstanceError::CapabilityOperationError
        | InstanceError::PrelaunchCommandError { .. }
        | InstanceError::Hook(_)
        | InstanceError::UpdateFailed { .. }
        | InstanceError::CredentialsError(_) => Kind::Internal,

        // As a refused command of a plugin call: the reason, and the words of the plugin that
        // reported it, pass on to the plugin that asked for the capability.
        InstanceError::PluginCommandNotApproved {
            reason, details, ..
        } => {
            return HostError::CommandNotApproved {
                reason: (*reason).into(),
                message: details.clone(),
            };
        }

        InstanceError::ContentDownloadError(failure) => return content_download(value, failure),
        InstanceError::ContentProviderError { reason } => return content_provider(value, reason),

        // Their own text: the wrapper's names only the step that failed.
        InstanceError::ProcessError(err) => return Kind::Internal.of(err),
        InstanceError::SettingsLoadError(err) => return settings(err),
        InstanceError::MinecraftError(err) => return minecraft(err),
    };

    kind.of(value)
}

/// A file or folder the storage looked for is not there: the plugin asked for something absent,
/// the disk is not broken.
fn storage_is_missing(value: &StorageError) -> bool {
    use std::io::ErrorKind::NotFound;

    match value {
        StorageError::Io(err)
        | StorageError::SharedIo(
            IoError::IoPathError { source: err, .. } | IoError::IoError(err),
        ) => err.kind() == NotFound,
        _ => false,
    }
}

fn content_download(value: &InstanceError, failure: &ContentDownloadFailure) -> HostError {
    let kind = match failure {
        ContentDownloadFailure::NoFiles { .. } | ContentDownloadFailure::NoVersions { .. } => {
            Kind::NotFound
        }
        ContentDownloadFailure::NotAModpackFile { .. } => Kind::InvalidArgument,
        ContentDownloadFailure::Download { source, .. } => {
            return http_or_internal(value, from_source(&**source));
        }
        ContentDownloadFailure::NoNetworkPolicy { .. } => Kind::Conflict,
    };

    kind.of(value)
}

fn content_provider(value: &InstanceError, rejection: &ContentProviderRejection) -> HostError {
    let kind = match rejection {
        ContentProviderRejection::InvalidRequest(_)
        | ContentProviderRejection::NotAModpack { .. } => Kind::InvalidArgument,

        ContentProviderRejection::PackOutsideProviderReach { .. }
        | ContentProviderRejection::NoAllowedDownload { .. } => Kind::AccessDenied,

        ContentProviderRejection::TooManyFiles { .. }
        | ContentProviderRejection::TooManyOverrides { .. }
        | ContentProviderRejection::PackIndexTooLarge { .. } => Kind::TooLarge,

        ContentProviderRejection::Request(source) => {
            return http_or_internal(value, from_source(&**source));
        }

        // What a provider plugin answered: not the caller's to fix.
        ContentProviderRejection::InvalidAnswer { .. } | ContentProviderRejection::Provider(_) => {
            Kind::PluginCallFailed
        }

        // A provider's answer, or a pack, that cannot be used.
        ContentProviderRejection::WrongTarget { .. }
        | ContentProviderRejection::ForeignPack { .. }
        | ContentProviderRejection::InvalidOverridesArchive(_)
        | ContentProviderRejection::VersionOfAnotherContent { .. }
        | ContentProviderRejection::NoGameVersion { .. }
        | ContentProviderRejection::UnknownContentType { .. }
        | ContentProviderRejection::ResponseInvalidId { .. }
        | ContentProviderRejection::ResponseInvalidUrl(_)
        | ContentProviderRejection::ResponseInvalidPagination(_)
        | ContentProviderRejection::PackUnreadable(_)
        | ContentProviderRejection::PackInvalidArchive(_)
        | ContentProviderRejection::PackNoIndex { .. }
        | ContentProviderRejection::PackIndexMalformed(_)
        | ContentProviderRejection::PackWrongGame { .. }
        | ContentProviderRejection::InvalidHash(_) => Kind::Internal,
    };

    kind.of(value)
}

fn java(value: &JavaApplicationError) -> HostError {
    let domain = match value {
        JavaApplicationError::Domain(err) => err,
        JavaApplicationError::DownloadFailed(err) => return request(err),
    };
    let kind = match domain {
        JavaDomainError::InvalidPath { .. }
        | JavaDomainError::EmptyPath
        | JavaDomainError::RelativePath { .. }
        | JavaDomainError::InvalidVersion { .. } => Kind::InvalidArgument,
        JavaDomainError::NotFound { .. } | JavaDomainError::VersionNotAvailable { .. } => {
            Kind::NotFound
        }
        JavaDomainError::Cancelled => Kind::Cancelled,
        JavaDomainError::PropertiesTimeout { .. } => Kind::TimedOut,
        JavaDomainError::FailedToGetProperties { .. }
        | JavaDomainError::VersionGetFailed { .. }
        | JavaDomainError::FailedToInstall { .. }
        | JavaDomainError::RemoveOldInstallationError { .. }
        | JavaDomainError::Storage(_) => Kind::Internal,
    };

    kind.of(domain)
}

fn minecraft(value: &MinecraftApplicationError) -> HostError {
    let domain = match value {
        MinecraftApplicationError::Domain(err) => err,
        MinecraftApplicationError::JavaError(err) => return java(err),
        MinecraftApplicationError::Storage(_) => return Kind::Internal.of(value),
    };
    let kind = match domain {
        MinecraftDomainError::LoaderVersionRequired => Kind::InvalidArgument,
        MinecraftDomainError::VersionNotFound { .. }
        | MinecraftDomainError::VersionForLoaderNotFound { .. }
        | MinecraftDomainError::LoaderNotFound { .. }
        | MinecraftDomainError::DefaultLoaderNotFound => Kind::NotFound,
        MinecraftDomainError::ProcessorFailed { .. }
        | MinecraftDomainError::PreLaunchFailed { .. }
        | MinecraftDomainError::PostLaunchFailed { .. }
        | MinecraftDomainError::PathNotFound { .. }
        | MinecraftDomainError::ParseFailed { .. }
        | MinecraftDomainError::StorageFailure(_)
        | MinecraftDomainError::UnsafeMetadataPath { .. }
        | MinecraftDomainError::InvalidLogConfigName { .. } => Kind::Internal,
    };

    kind.of(domain)
}

fn plugin(value: &PluginError) -> HostError {
    let kind = match value {
        PluginError::IdMismatch { .. }
        | PluginError::InvalidAllowedPath { .. }
        | PluginError::InvalidAllowedHost { .. }
        | PluginError::InvalidLocalHost { .. }
        | PluginError::InvalidAllowedHostEntry { .. }
        | PluginError::CommandApprovalOnceOnly { .. } => Kind::InvalidArgument,

        PluginError::NotFound { .. } | PluginError::CommandApprovalNotFound { .. } => {
            Kind::NotFound
        }

        PluginError::AccessViolation { .. } | PluginError::ForeignProvider { .. } => {
            Kind::AccessDenied
        }

        PluginError::AlreadyLoaded { .. }
        | PluginError::LoadingInProgress { .. }
        | PluginError::UnloadingInProgress { .. }
        | PluginError::AlreadyUnloaded { .. }
        | PluginError::Busy { .. }
        | PluginError::Stopped { .. } => Kind::Conflict,

        PluginError::LoadFailed { .. }
        | PluginError::IncompatibleApiVersion { .. }
        | PluginError::Manifest(_)
        | PluginError::LoaderNotFound { .. }
        | PluginError::CapabilityRegistrationFailed { .. }
        | PluginError::CapabilityCancelRegistrationFailed { .. }
        | PluginError::HashComputationFailed
        | PluginError::Storage(_) => Kind::Internal,

        PluginError::CommandNotApproved { reason, .. } => {
            return HostError::CommandNotApproved {
                reason: (*reason).into(),
                message: value.to_string(),
            };
        }
        PluginError::FunctionCallFailed { error, .. } => return plugin_call(value, error),
        PluginError::Settings(err) => return settings(err),
        PluginError::Provider(err) => return plugin_provider(value, err),
    };

    kind.of(value)
}

/// A call into another plugin that failed. A refused command keeps its reason, and the words
/// of the plugin that reported it, so that a plugin passing the error on passes them on too. A
/// host refused to the other plugin is only a failed call: as `http` / `hostNotAllowed` it would
/// read as a refusal of the caller's own, and the user would be offered to allow the host to it.
fn plugin_call(value: &PluginError, error: &PluginCallError) -> HostError {
    match error {
        PluginCallError::CommandNotApproved { reason, details } => HostError::CommandNotApproved {
            reason: (*reason).into(),
            message: details.clone(),
        },
        PluginCallError::HostNotAllowed { .. }
        | PluginCallError::AddressBlocked { .. }
        | PluginCallError::MissingFunction
        | PluginCallError::Timeout
        | PluginCallError::OutOfFuel
        | PluginCallError::OutOfMemory
        | PluginCallError::Trap(_)
        | PluginCallError::NonZeroExit(_)
        | PluginCallError::Plugin(_)
        | PluginCallError::Codec(_)
        | PluginCallError::Reentrant => Kind::PluginCallFailed.of(value),
    }
}

/// A command `run_command` was allowed to run. Only running out of time is the plugin's to know
/// about; the rest is the host failing to start or watch the process.
fn plugin_command(value: &PluginCommandError) -> HostError {
    let kind = match value {
        PluginCommandError::TimedOut { .. } => Kind::TimedOut,
        PluginCommandError::Spawn { .. }
        | PluginCommandError::OutputNotCaptured { .. }
        | PluginCommandError::ReadOutput { .. } => Kind::Internal,
    };

    kind.of(value)
}

/// Only the user installs or updates a plugin; a host function never gets here.
fn plugin_provider(value: &PluginError, provider: &PluginProviderError) -> HostError {
    let kind = match provider {
        PluginProviderError::ManifestNotFound { .. }
        | PluginProviderError::InvalidManifestFormat { .. }
        | PluginProviderError::ExtractionFailed { .. }
        | PluginProviderError::InvalidExtractionFormat
        | PluginProviderError::FileExtractionFailed { .. }
        | PluginProviderError::Manifest(_) => Kind::InvalidArgument,
        PluginProviderError::ProviderNoAssets { .. }
        | PluginProviderError::NoRemoteSource { .. } => Kind::NotFound,
        PluginProviderError::ArchiveChangedSincePreview => Kind::Conflict,
        PluginProviderError::ProviderFetchError { reason, .. }
        | PluginProviderError::DownloadFailed { reason, .. } => {
            return provider_failure(value, reason);
        }
        PluginProviderError::ProviderRateLimited { status, .. } => {
            return http(value, HttpErrorDto::Status { status: *status });
        }
        PluginProviderError::Storage(_) => Kind::Internal,
    };

    kind.of(value)
}

fn provider_failure(value: &PluginError, failure: &ProviderFailure) -> HostError {
    let error = match failure {
        ProviderFailure::Request(source) => return http_or_internal(value, from_source(&**source)),
        ProviderFailure::Status { status, .. } => HttpErrorDto::Status { status: *status },
        ProviderFailure::TooLarge { limit } => HttpErrorDto::ResponseTooLarge { limit: *limit },
        ProviderFailure::UrlNotAllowed => HttpErrorDto::InvalidResponse {
            reason: failure.to_string(),
        },
        ProviderFailure::InvalidUrl { .. } => return Kind::InvalidArgument.of(value),
        ProviderFailure::NoReleases | ProviderFailure::NotRegistered => {
            return Kind::NotFound.of(value);
        }
    };

    http(value, error)
}

fn settings(value: &SettingsError) -> HostError {
    let kind = match value {
        SettingsError::InvalidPlatformProvider { .. }
        | SettingsError::InvalidMaxConcurrentDownloads
        | SettingsError::InvalidMemory => Kind::InvalidArgument,
        // The launcher's own settings: nothing the plugin named is missing.
        SettingsError::NotFound | SettingsError::Storage(_) => Kind::Internal,
    };

    kind.of(value)
}

fn request(value: &RequestError) -> HostError {
    http_or_internal(value, request_kind(value))
}

/// A network request failed: `message` is the text of `error`, `kind` what the plugin needs to
/// tell how.
fn http(error: &impl Display, kind: HttpErrorDto) -> HostError {
    HostError::Http {
        message: error.to_string(),
        error: kind,
    }
}

/// [`http`], or `internal` when there is no `kind`: the host failed on its own while it made the
/// request (the disk, the download slots, the cache).
fn http_or_internal(error: &impl Display, kind: Option<HttpErrorDto>) -> HostError {
    match kind {
        Some(kind) => http(error, kind),
        None => Kind::Internal.of(error),
    }
}

/// A request of the core's own client; `None` when it failed in the host, not on the network.
fn request_kind(value: &RequestError) -> Option<HttpErrorDto> {
    let reason = || value.to_string();
    let kind = match value {
        RequestError::RequestSend(_) | RequestError::Middleware(_) => {
            HttpErrorDto::Network { reason: reason() }
        }
        RequestError::Status { status, .. } => HttpErrorDto::Status { status: *status },
        // The body that came is not the file the expected hash names.
        RequestError::Hash { .. } => HttpErrorDto::HashMismatch,
        RequestError::Json(_) | RequestError::Toml(_) | RequestError::Parse(_) => {
            HttpErrorDto::InvalidResponse { reason: reason() }
        }
        RequestError::TooLarge { limit } => HttpErrorDto::ResponseTooLarge { limit: *limit },
        RequestError::Acquire(_) | RequestError::Io { .. } => return None,
    };

    Some(kind)
}

/// A request `http_get` or `http_post_json` makes for a plugin, or a hop of a download.
fn plugin_http(value: &PluginHttpError) -> HttpErrorDto {
    let reason = || value.to_string();
    match value {
        PluginHttpError::HostNotAllowed { host } => {
            HttpErrorDto::HostNotAllowed { host: host.clone() }
        }
        PluginHttpError::BlockedAddress { host, port, .. } => HttpErrorDto::AddressBlocked {
            host: host.clone(),
            port: *port,
        },
        PluginHttpError::InvalidUrl { .. }
        | PluginHttpError::UnsupportedScheme { .. }
        | PluginHttpError::MissingHost { .. }
        | PluginHttpError::InvalidJsonBody { .. }
        | PluginHttpError::RequestBodyTooLarge { .. }
        | PluginHttpError::ForbiddenHeader { .. }
        | PluginHttpError::InvalidHeader { .. } => {
            HttpErrorDto::InvalidRequest { reason: reason() }
        }
        PluginHttpError::RedirectNotAllowed { .. }
        | PluginHttpError::InvalidRedirect { .. }
        | PluginHttpError::SchemeDowngrade { .. }
        | PluginHttpError::TooManyRedirects { .. } => HttpErrorDto::Redirect { reason: reason() },
        PluginHttpError::ResponseTooLarge { limit } => HttpErrorDto::ResponseTooLarge {
            limit: u64::try_from(*limit).unwrap_or(u64::MAX),
        },
        // A name with no address is one that could not be resolved: no grant would help.
        PluginHttpError::ResolveFailed { .. }
        | PluginHttpError::NoAddresses { .. }
        | PluginHttpError::RequestFailed { .. } => HttpErrorDto::Network { reason: reason() },
    }
}

/// A download made for a plugin; `None` when the host could not use the plugin's cache.
fn plugin_download(value: &PluginDownloadError) -> Option<HttpErrorDto> {
    match value {
        // The hash the plugin sent is no digest at all: not a mismatch.
        PluginDownloadError::InvalidHash(err) => Some(HttpErrorDto::InvalidRequest {
            reason: err.to_string(),
        }),
        PluginDownloadError::Http(err) => Some(plugin_http(err)),
        PluginDownloadError::Request(err) => request_kind(err),
        PluginDownloadError::OutOfTime(_) => Some(HttpErrorDto::Network {
            reason: value.to_string(),
        }),
        PluginDownloadError::UnusableCache { .. } => None,
    }
}

/// A request failure kept as an opaque source: told by the error under it when it is one of the
/// core's, otherwise a request that got no usable answer.
fn from_source(source: &(dyn Error + 'static)) -> Option<HttpErrorDto> {
    if let Some(err) = source.downcast_ref::<RequestError>() {
        return request_kind(err);
    }
    if let Some(err) = source.downcast_ref::<PluginDownloadError>() {
        return plugin_download(err);
    }
    if let Some(err) = source.downcast_ref::<PluginHttpError>() {
        return Some(plugin_http(err));
    }

    Some(HttpErrorDto::Network {
        reason: source.to_string(),
    })
}
impl From<CommandApprovalRefusal> for CommandRefusalDto {
    fn from(value: CommandApprovalRefusal) -> Self {
        match value {
            CommandApprovalRefusal::Denied => Self::Denied,
            CommandApprovalRefusal::TimedOut => Self::TimedOut,
            CommandApprovalRefusal::Cancelled => Self::Cancelled,
            CommandApprovalRefusal::JarChanged => Self::JarChanged,
        }
    }
}

impl TryFrom<CommandRefusalDto> for CommandApprovalRefusal {
    type Error = super::instance::UnknownValue;

    fn try_from(value: CommandRefusalDto) -> Result<Self, Self::Error> {
        match value {
            CommandRefusalDto::Denied => Ok(Self::Denied),
            CommandRefusalDto::TimedOut => Ok(Self::TimedOut),
            CommandRefusalDto::Cancelled => Ok(Self::Cancelled),
            CommandRefusalDto::JarChanged => Ok(Self::JarChanged),
            CommandRefusalDto::Unknown => Err(super::instance::UnknownValue {
                kind: "command refusal",
                expected: &["denied", "timedOut", "cancelled", "jarChanged"],
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::features::{
        auth::{AuthApplicationError, AuthDomainError},
        events::EventError,
        file_watcher::FileWatcherError,
        game_log::GameLogError,
        instance::{PageSize, PluginId},
        process::ProcessError,
    };
    use crate::shared::hash::{FileHash, HashAlgorithm};

    fn host(error: impl Into<ErrorKind>) -> HostError {
        HostError::from(&error.into())
    }

    fn message(error: impl Into<ErrorKind>) -> String {
        error.into().to_string()
    }

    #[test]
    fn should_tell_not_found_when_the_storage_has_no_such_file() {
        let error = InstanceError::storage(std::io::Error::from(std::io::ErrorKind::NotFound));

        assert_eq!(
            host(error.clone()),
            HostError::NotFound {
                message: message(error)
            }
        );
    }

    #[test]
    fn should_tell_internal_when_the_storage_failed_for_another_reason() {
        let error =
            InstanceError::storage(std::io::Error::from(std::io::ErrorKind::PermissionDenied));

        assert!(matches!(host(error), HostError::Internal { .. }));
    }

    #[test]
    fn should_tell_an_invalid_argument_when_a_value_the_plugin_sent_is_refused() {
        let error = InstanceError::from(PageSize::parse(0).unwrap_err());

        assert_eq!(
            host(error.clone()),
            HostError::InvalidArgument {
                message: message(error)
            }
        );
    }

    #[test]
    fn should_tell_not_found_when_the_instance_does_not_exist() {
        let error = InstanceError::NotFound {
            instance_id: "abc".to_owned(),
        };

        assert_eq!(
            host(error),
            HostError::NotFound {
                message: "Instance with id \"abc\" not found".to_owned()
            }
        );
    }

    #[test]
    fn should_tell_access_denied_when_a_path_escapes_the_instance() {
        let error = InstanceError::PathEscapesInstance {
            path: "../x".to_owned(),
            instance_id: "inst".to_owned(),
        };

        assert_eq!(
            host(error.clone()),
            HostError::AccessDenied {
                message: message(error)
            }
        );
    }

    #[test]
    fn should_tell_timed_out_when_a_command_ran_past_its_time() {
        let error = PluginCommandError::TimedOut {
            program: "java".to_owned(),
            seconds: 900,
        };

        assert_eq!(
            host(error),
            HostError::TimedOut {
                message: "Command `java` exceeded the 900 second time limit and was killed"
                    .to_owned()
            }
        );
    }

    #[test]
    fn should_tell_internal_when_a_command_could_not_be_started() {
        let error = PluginCommandError::Spawn {
            program: "java".to_owned(),
            reason: "denied".to_owned(),
        };

        assert_eq!(
            host(error),
            HostError::Internal {
                message: "Failed to run `java`: denied".to_owned()
            }
        );
    }

    #[rstest]
    #[case(CommandApprovalRefusal::Denied)]
    #[case(CommandApprovalRefusal::TimedOut)]
    #[case(CommandApprovalRefusal::Cancelled)]
    #[case(CommandApprovalRefusal::JarChanged)]
    fn should_hand_a_refused_command_over_with_its_reason(#[case] reason: CommandApprovalRefusal) {
        let error = PluginError::CommandNotApproved {
            plugin_id: PluginId::from_static("packwiz"),
            reason,
        };

        let HostError::CommandNotApproved {
            reason: dto,
            message: text,
        } = host(error.clone())
        else {
            panic!("expected CommandNotApproved");
        };
        assert_eq!(text, message(error));
        // The word on the wire is the one `as_wire` names, which the docs list.
        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            serde_json::json!(reason.as_wire())
        );
        assert_eq!(CommandApprovalRefusal::try_from(dto), Ok(reason));
    }

    #[test]
    fn should_keep_the_words_of_the_other_plugin_when_its_command_was_refused() {
        let error = PluginError::FunctionCallFailed {
            function_name: "install".to_owned(),
            plugin_id: PluginId::from_static("b"),
            error: PluginCallError::CommandNotApproved {
                reason: CommandApprovalRefusal::Cancelled,
                details: "b said so".to_owned(),
            },
        };

        assert_eq!(
            host(error),
            HostError::CommandNotApproved {
                reason: CommandRefusalDto::Cancelled,
                message: "b said so".to_owned(),
            }
        );
    }

    /// A command refused to the plugin behind a capability passes on the way a command refused
    /// in a call of that plugin does.
    #[test]
    fn should_keep_the_words_of_the_plugin_behind_a_capability_when_its_command_was_refused() {
        let error = InstanceError::PluginCommandNotApproved {
            plugin_id: "b".to_owned(),
            reason: CommandApprovalRefusal::Denied,
            details: "b said so".to_owned(),
        };

        assert_eq!(
            host(error),
            HostError::CommandNotApproved {
                reason: CommandRefusalDto::Denied,
                message: "b said so".to_owned(),
            }
        );
    }

    /// A request the host makes for a plugin keeps its kind, and the error's own text.
    #[rstest]
    #[case::refused_host(
        PluginHttpError::HostNotAllowed { host: "cdn.example".to_owned() }.into(),
        HttpErrorDto::HostNotAllowed { host: "cdn.example".to_owned() }
    )]
    #[case::blocked_address(
        PluginHttpError::BlockedAddress {
            host: "intranet.example".to_owned(),
            port: 8080,
            ip: std::net::Ipv4Addr::new(10, 0, 0, 1).into(),
        }
        .into(),
        HttpErrorDto::AddressBlocked { host: "intranet.example".to_owned(), port: 8080 }
    )]
    #[case::no_addresses(
        PluginHttpError::NoAddresses { host: "cdn.example".to_owned() }.into(),
        HttpErrorDto::Network {
            reason: "Host `cdn.example` resolved to no addresses".to_owned()
        }
    )]
    #[case::redirect_of_a_download(
        PluginDownloadError::Http(PluginHttpError::TooManyRedirects { limit: 5 }).into(),
        HttpErrorDto::Redirect { reason: "Too many redirects: more than 5".to_owned() }
    )]
    #[case::response_too_large(
        PluginHttpError::ResponseTooLarge { limit: 16 }.into(),
        HttpErrorDto::ResponseTooLarge { limit: 16 }
    )]
    #[case::download_too_large(
        PluginDownloadError::Request(RequestError::TooLarge { limit: 7 }).into(),
        HttpErrorDto::ResponseTooLarge { limit: 7 }
    )]
    #[case::core_response_too_large(
        RequestError::TooLarge { limit: 7 }.into(),
        HttpErrorDto::ResponseTooLarge { limit: 7 }
    )]
    #[case::network(
        PluginHttpError::ResolveFailed {
            host: "cdn.example".to_owned(),
            reason: "no such host".to_owned(),
        }
        .into(),
        HttpErrorDto::Network {
            reason: "Failed to resolve host `cdn.example`: no such host".to_owned()
        }
    )]
    fn should_tell_http_with_its_kind_when_a_plugin_request_failed(
        #[case] error: ErrorKind,
        #[case] kind: HttpErrorDto,
    ) {
        assert_eq!(
            host(error.clone()),
            HostError::Http {
                message: error.to_string(),
                error: kind,
            }
        );
    }

    #[test]
    fn should_tell_a_hash_mismatch_from_a_malformed_hash_when_a_download_failed() {
        let malformed = FileHash::new(HashAlgorithm::Sha1, "zz").unwrap_err();
        let mismatch = PluginDownloadError::Request(RequestError::Hash {
            actual: "aa".to_owned(),
            expected: "bb".to_owned(),
        });

        assert_eq!(
            host(mismatch.clone()),
            HostError::Http {
                message: message(mismatch),
                error: HttpErrorDto::HashMismatch,
            }
        );
        assert_eq!(
            host(PluginDownloadError::InvalidHash(malformed.clone())),
            HostError::Http {
                message: message(PluginDownloadError::InvalidHash(malformed.clone())),
                error: HttpErrorDto::InvalidRequest {
                    reason: malformed.to_string()
                },
            }
        );
    }

    #[test]
    fn should_tell_a_conflict_when_the_instance_is_still_installing() {
        let error = InstanceError::InstanceStillInstalling {
            instance_id: "inst".to_owned(),
        };

        assert_eq!(
            host(error.clone()),
            HostError::Conflict {
                message: message(error)
            }
        );
    }

    #[test]
    fn should_tell_cancelled_when_the_work_was_called_off() {
        assert_eq!(
            host(InstanceError::Cancelled),
            HostError::Cancelled {
                message: message(InstanceError::Cancelled)
            }
        );
    }

    #[test]
    fn should_tell_timed_out_when_java_did_not_answer_in_time() {
        let error = JavaApplicationError::Domain(JavaDomainError::PropertiesTimeout {
            path: "java".into(),
            seconds: 5,
        });

        assert_eq!(
            host(error.clone()),
            HostError::TimedOut {
                message: message(error)
            }
        );
    }

    #[test]
    fn should_tell_a_failed_plugin_call_when_the_other_plugin_hit_a_limit() {
        let error = PluginError::FunctionCallFailed {
            function_name: "install".to_owned(),
            plugin_id: PluginId::from_static("packwiz"),
            error: PluginCallError::Timeout,
        };

        assert_eq!(
            host(error.clone()),
            HostError::PluginCallFailed {
                message: message(error)
            }
        );
    }

    /// A host refused to the plugin that was called is not the caller's own refusal: told as
    /// `http` / `hostNotAllowed`, the caller would pass it back as its own.
    #[test]
    fn should_tell_a_failed_plugin_call_when_a_host_was_refused_to_the_other_plugin() {
        let call = PluginError::FunctionCallFailed {
            function_name: "install".to_owned(),
            plugin_id: PluginId::from_static("b"),
            error: PluginCallError::HostNotAllowed {
                host: "cdn.example".to_owned(),
                details: "b refused".to_owned(),
            },
        };
        let capability = InstanceError::PluginHostNotAllowed {
            plugin_id: "b".to_owned(),
            host: "cdn.example".to_owned(),
            details: "b refused".to_owned(),
        };

        assert_eq!(
            host(call.clone()),
            HostError::PluginCallFailed {
                message: message(call)
            }
        );
        assert_eq!(
            host(capability.clone()),
            HostError::PluginCallFailed {
                message: message(capability)
            }
        );
    }

    /// The same for a local address refused to the other plugin: the caller is not offered a
    /// grant for a plugin it is not.
    #[test]
    fn should_tell_a_failed_plugin_call_when_an_address_was_blocked_for_the_other_plugin() {
        let call = PluginError::FunctionCallFailed {
            function_name: "install".to_owned(),
            plugin_id: PluginId::from_static("b"),
            error: PluginCallError::AddressBlocked {
                host: "nas.lan".to_owned(),
                port: 80,
                details: "b refused".to_owned(),
            },
        };
        let capability = InstanceError::PluginAddressBlocked {
            plugin_id: "b".to_owned(),
            host: "nas.lan".to_owned(),
            port: 80,
            details: "b refused".to_owned(),
        };

        for error in [ErrorKind::from(call), ErrorKind::from(capability)] {
            assert_eq!(
                host(error.clone()),
                HostError::PluginCallFailed {
                    message: message(error)
                }
            );
        }
    }

    #[test]
    fn should_tell_http_when_a_request_failed_with_a_status() {
        let error = JavaApplicationError::DownloadFailed(RequestError::Status {
            status: 404,
            url: "https://example.com/jre".to_owned(),
        });

        assert_eq!(
            host(error),
            HostError::Http {
                message: "Unexpected HTTP status 404 from `https://example.com/jre`".to_owned(),
                error: HttpErrorDto::Status { status: 404 },
            }
        );
    }

    #[test]
    fn should_tell_internal_without_the_display_prefix_when_the_core_failed() {
        assert_eq!(
            host(ErrorKind::CoreError("disk full".to_owned())),
            HostError::Internal {
                message: "disk full".to_owned()
            }
        );
    }

    #[test]
    fn should_classify_a_wrapped_error_by_itself_with_its_own_text() {
        let wrapped = MinecraftApplicationError::Domain(MinecraftDomainError::VersionNotFound {
            version: "1.20".to_owned(),
        });

        assert_eq!(
            host(InstanceError::MinecraftError(wrapped.clone())),
            HostError::NotFound {
                message: message(wrapped)
            }
        );
    }

    /// No host function reaches these features; should one, it still maps without a kind of
    /// its own.
    #[rstest]
    #[case(AuthApplicationError::Domain(AuthDomainError::TokenExpired).into())]
    #[case(EventError::NotInitialized.into())]
    #[case(FileWatcherError::WatchNotFound.into())]
    #[case(GameLogError::SourceNotFound { source_id: "latest".to_owned() }.into())]
    #[case(ProcessError::KillError { id: "p1".to_owned() }.into())]
    fn should_tell_internal_when_a_feature_no_host_function_reaches_failed(
        #[case] error: ErrorKind,
    ) {
        assert_eq!(
            host(error.clone()),
            HostError::Internal {
                message: error.to_string()
            }
        );
    }
}
