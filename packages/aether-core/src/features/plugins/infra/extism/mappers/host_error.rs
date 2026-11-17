//! Maps the core's errors onto the [`HostError`] a plugin receives from a host function.
//!
//! This is the one place that decides what kind an error is for a plugin: what it can do about
//! it, not which feature failed. Each enum is matched variant by variant, so a new variant has
//! to be placed here before it compiles. An error that wraps another feature's error is
//! classified by the wrapped one. The message is the core's own text of the error.

use std::fmt::Display;

use aether_core_plugin_api::v0::{CommandRefusalDto, HostError};

use crate::{
    ErrorKind,
    features::{
        instance::{ContentDownloadFailure, ContentProviderRejection, InstanceError},
        java::{JavaApplicationError, JavaDomainError},
        minecraft::{MinecraftApplicationError, MinecraftDomainError},
        plugins::{CommandApprovalRefusal, PluginCallError, PluginError, PluginProviderError},
        settings::SettingsError,
    },
    shared::request_client::RequestError,
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
    Http,
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
            Self::Http => HostError::Http { message },
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

        InstanceError::ImportFailed { .. } => Kind::PluginCallFailed,

        InstanceError::Storage(_)
        | InstanceError::InvalidRecord { .. }
        | InstanceError::ContentStateChangeFailed { .. }
        | InstanceError::CapabilityOperationError
        | InstanceError::PrelaunchCommandError { .. }
        | InstanceError::Hook(_)
        | InstanceError::UpdateFailed { .. }
        | InstanceError::ProcessError(_)
        | InstanceError::CredentialsError(_) => Kind::Internal,

        InstanceError::ContentDownloadError(failure) => content_download(failure),
        InstanceError::ContentProviderError { reason } => content_provider(reason),

        // Their own text: the wrapper's names only the step that failed.
        InstanceError::SettingsLoadError(err) => return settings(err),
        InstanceError::MinecraftError(err) => return minecraft(err),
    };

    kind.of(value)
}

fn content_download(value: &ContentDownloadFailure) -> Kind {
    match value {
        ContentDownloadFailure::NoFiles { .. } | ContentDownloadFailure::NoVersions { .. } => {
            Kind::NotFound
        }
        ContentDownloadFailure::NotAModpackFile { .. } => Kind::InvalidArgument,
        ContentDownloadFailure::Download { .. } => Kind::Http,
        ContentDownloadFailure::NoNetworkPolicy { .. } => Kind::Conflict,
    }
}

fn content_provider(value: &ContentProviderRejection) -> Kind {
    match value {
        ContentProviderRejection::InvalidRequest(_)
        | ContentProviderRejection::NotAModpack { .. } => Kind::InvalidArgument,

        ContentProviderRejection::PackOutsideProviderReach { .. }
        | ContentProviderRejection::NoAllowedDownload { .. } => Kind::AccessDenied,

        ContentProviderRejection::TooManyFiles { .. }
        | ContentProviderRejection::TooManyOverrides { .. }
        | ContentProviderRejection::PackIndexTooLarge { .. } => Kind::TooLarge,

        ContentProviderRejection::Request(_) => Kind::Http,

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
    }
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
        | PluginError::InvalidAllowedHostEntry { .. }
        | PluginError::CommandApprovalOnceOnly { .. } => Kind::InvalidArgument,

        PluginError::NotFound { .. } | PluginError::CommandApprovalNotFound { .. } => {
            Kind::NotFound
        }

        PluginError::AccessViolation { .. } => Kind::AccessDenied,

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
        PluginError::Provider(err) => plugin_provider(err),
    };

    kind.of(value)
}

/// A call into another plugin that failed. A refused command keeps its reason, and the words
/// of the plugin that reported it, so that a plugin passing the error on passes them on too.
fn plugin_call(value: &PluginError, error: &PluginCallError) -> HostError {
    match error {
        PluginCallError::CommandNotApproved { reason, details } => HostError::CommandNotApproved {
            reason: (*reason).into(),
            message: details.clone(),
        },
        PluginCallError::MissingFunction
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

/// Only the user installs or updates a plugin; a host function never gets here.
fn plugin_provider(value: &PluginProviderError) -> Kind {
    match value {
        PluginProviderError::ManifestNotFound { .. }
        | PluginProviderError::InvalidManifestFormat { .. }
        | PluginProviderError::ExtractionFailed { .. }
        | PluginProviderError::InvalidExtractionFormat
        | PluginProviderError::FileExtractionFailed { .. }
        | PluginProviderError::Manifest(_) => Kind::InvalidArgument,
        PluginProviderError::ProviderNoAssets { .. }
        | PluginProviderError::NoRemoteSource { .. } => Kind::NotFound,
        PluginProviderError::ArchiveChangedSincePreview => Kind::Conflict,
        PluginProviderError::ProviderFetchError { .. }
        | PluginProviderError::DownloadFailed { .. }
        | PluginProviderError::ProviderRateLimited { .. } => Kind::Http,
        PluginProviderError::Storage(_) => Kind::Internal,
    }
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
    let kind = match value {
        RequestError::RequestSend(_)
        | RequestError::Middleware(_)
        | RequestError::Status { .. }
        | RequestError::Hash { .. }
        | RequestError::Json(_)
        | RequestError::Toml(_)
        | RequestError::Parse(_) => Kind::Http,
        RequestError::TooLarge { .. } => Kind::TooLarge,
        RequestError::Acquire(_) | RequestError::Io { .. } => Kind::Internal,
    };

    kind.of(value)
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

    fn host(error: impl Into<ErrorKind>) -> HostError {
        HostError::from(&error.into())
    }

    fn message(error: impl Into<ErrorKind>) -> String {
        error.into().to_string()
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

    #[test]
    fn should_tell_too_large_when_a_response_is_over_the_limit() {
        let error = RequestError::TooLarge { limit: 7 };

        assert_eq!(
            host(error.clone()),
            HostError::TooLarge {
                message: message(error)
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

    #[test]
    fn should_tell_http_when_a_request_failed_with_a_status() {
        let error = JavaApplicationError::DownloadFailed(RequestError::Status {
            status: 404,
            url: "https://example.com/jre".to_owned(),
        });

        assert_eq!(
            host(error),
            HostError::Http {
                message: "Unexpected HTTP status 404 from `https://example.com/jre`".to_owned()
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
