use aether_core::features::{
    instance::{
        InstanceError, InstanceField, InstanceValidationErrorReason, PaginationField,
        PaginationRejection, StorageError,
    },
    minecraft::{GameVersionRejection, LoaderVersionIdRejection},
};
use aether_core::shared::safe_path::PathRejection;
use aether_core::shared::serializable_command::CommandParseError;
use serde::Serialize;
use specta::Type;
use uuid::Uuid;

use super::{CauseDto, FrontendError};
use crate::features::instance::infra::ContentTypeDto;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstanceErrorDto {
    Storage {
        cause: CauseDto,
    },
    NotFound {
        instance_id: String,
    },
    ContentProviderNotFound {
        plugin_id: String,
        capability_id: String,
    },
    CapabilityOperationError,
    InstanceStillInstalling {
        instance_id: String,
    },
    InstanceAlreadyRunning {
        instance_id: String,
        process_id: Uuid,
    },
    PrelaunchCommandError {
        code: i32,
    },
    PrelaunchCommandInvalid {
        reason: CommandParseReasonDto,
    },
    PrelaunchCommandSpawnFailed {
        cause: CauseDto,
    },
    PrelaunchCommandTimeout {
        // Formatted before it crosses the boundary: no `u64` in bindings.
        seconds: String,
    },
    ValidationError {
        field: InstanceFieldDto,
        reason: InstanceValidationErrorReasonDto,
    },
    ImporterNotFound {
        importer_id: String,
    },
    ImportFailed {
        plugin_id: String,
        capability_id: String,
        cause: CauseDto,
    },
    PluginNotRunning {
        plugin_id: String,
    },
    CachesBeingCleared,
    IconTooLarge {
        // As for `FileTooLarge`, formatted before it crosses the boundary: no `u64` in bindings.
        limit: String,
    },
    IconNotSupported,
    Cancelled,
    PackInfoNotFound,
    UpdaterNotFound {
        modpack_id: String,
    },
    UpdateFailed {
        modpack_id: String,
    },
    NotBroken {
        instance_id: String,
    },
    UnmanagedInstance {
        instance_id: String,
    },
    ContentDuplication {
        content_path: String,
    },
    ContentFilename {
        path: String,
    },
    InvalidRelativePath {
        path: String,
        reason: PathRejectionDto,
    },
    InvalidContentId {
        kind: String,
        value: String,
    },
    InvalidGameVersion {
        value: String,
        reason: GameVersionReasonDto,
    },
    InvalidPagination {
        field: PaginationFieldDto,
        // Formatted before it crosses the boundary: no `i64` in bindings.
        value: String,
        reason: PaginationReasonDto,
    },
    InvalidLoaderVersion {
        value: String,
        reason: LoaderVersionReasonDto,
    },
    PathEscapesInstance {
        path: String,
        instance_id: String,
    },
    FileTooLarge {
        // Specta refuses to export `u64` to TypeScript, and these two only ever end up in an
        // error message, so they cross the boundary already formatted.
        path: String,
        size: String,
        limit: String,
    },
    ContentDownloadError(String),
    ContentProviderError {
        reason: String,
    },
    ContentForGameVersionNotFound {
        game_version: String,
    },
    UnsupportedContentType {
        content_type: ContentTypeDto,
    },
    UnsupportedOperation {
        provider_id: String,
        operation: String,
    },
    PlatformProviderNotFound {
        platform: String,
    },
    PlatformProviderNotChosen {
        platform: String,
    },
    GameChangeNotAllowed {
        instance_id: String,
        provider_id: String,
        content_id: String,
    },
}

impl From<&InstanceError> for FrontendError {
    /// An error of another domain, wrapped by the instance one, is reported as its own: the
    /// frontend translates it by what failed, not by who was asking.
    fn from(value: &InstanceError) -> Self {
        match InstanceErrorDto::own(value) {
            Ok(dto) => Self::Instance(dto),
            Err(lifted) => *lifted,
        }
    }
}

impl From<&StorageError> for CauseDto {
    fn from(value: &StorageError) -> Self {
        match value {
            StorageError::Io(err) => err.into(),
            StorageError::SharedIo(err) => err.into(),
            StorageError::Database(err) => err.into(),
            StorageError::Other(details) => Self::unknown(details),
        }
    }
}

impl InstanceErrorDto {
    /// The DTO of an error the instance feature raises itself; an error of another domain it
    /// wraps comes back as that domain's own `FrontendError`.
    // One flat arm per variant; splitting an exhaustive mapping only hides which ones are covered.
    #[allow(clippy::too_many_lines)]
    fn own(value: &InstanceError) -> Result<Self, Box<FrontendError>> {
        Ok(match value {
            InstanceError::Storage(err) => Self::Storage { cause: err.into() },
            InstanceError::NotFound { instance_id } => Self::NotFound {
                instance_id: instance_id.clone(),
            },
            InstanceError::ContentProviderNotFound {
                plugin_id,
                capability_id,
            } => Self::ContentProviderNotFound {
                plugin_id: plugin_id.clone(),
                capability_id: capability_id.clone(),
            },
            InstanceError::CapabilityOperationError => Self::CapabilityOperationError,
            InstanceError::InstanceStillInstalling { instance_id } => {
                Self::InstanceStillInstalling {
                    instance_id: instance_id.clone(),
                }
            }
            InstanceError::InstanceAlreadyRunning {
                instance_id,
                process_id,
            } => Self::InstanceAlreadyRunning {
                instance_id: instance_id.clone(),
                process_id: *process_id,
            },
            InstanceError::PrelaunchCommandError { code } => {
                Self::PrelaunchCommandError { code: *code }
            }
            InstanceError::PrelaunchCommandInvalid { reason } => Self::PrelaunchCommandInvalid {
                reason: (*reason).into(),
            },
            InstanceError::PrelaunchCommandSpawnFailed { source, .. } => {
                Self::PrelaunchCommandSpawnFailed {
                    cause: source.into(),
                }
            }
            InstanceError::PrelaunchCommandTimeout { seconds } => Self::PrelaunchCommandTimeout {
                seconds: seconds.to_string(),
            },
            InstanceError::ValidationError { field, reason } => Self::ValidationError {
                field: (*field).into(),
                reason: reason.clone().into(),
            },
            InstanceError::ImporterNotFound { importer_id } => Self::ImporterNotFound {
                importer_id: importer_id.clone(),
            },
            InstanceError::ImportFailed {
                plugin_id,
                capability_id,
                reason,
            } => Self::ImportFailed {
                plugin_id: plugin_id.clone(),
                capability_id: capability_id.clone(),
                // The importer is a plugin: its text is not ours to categorize.
                cause: CauseDto::unknown(reason),
            },
            InstanceError::PluginNotRunning { plugin_id } => Self::PluginNotRunning {
                plugin_id: plugin_id.clone(),
            },
            InstanceError::CachesBeingCleared => Self::CachesBeingCleared,
            InstanceError::IconTooLarge { limit_bytes } => Self::IconTooLarge {
                limit: limit_bytes.to_string(),
            },
            InstanceError::IconNotSupported => Self::IconNotSupported,
            InstanceError::Cancelled => Self::Cancelled,
            InstanceError::PackInfoNotFound => Self::PackInfoNotFound,
            InstanceError::UpdaterNotFound { modpack_id } => Self::UpdaterNotFound {
                modpack_id: modpack_id.clone(),
            },
            InstanceError::UpdateFailed { modpack_id } => Self::UpdateFailed {
                modpack_id: modpack_id.clone(),
            },
            InstanceError::NotBroken { instance_id } => Self::NotBroken {
                instance_id: instance_id.clone(),
            },
            InstanceError::UnmanagedInstance { instance_id } => Self::UnmanagedInstance {
                instance_id: instance_id.clone(),
            },
            InstanceError::InvalidRelativePath { path, reason } => Self::InvalidRelativePath {
                path: path.clone(),
                reason: (*reason).into(),
            },
            InstanceError::InvalidContentId { kind, value } => Self::InvalidContentId {
                kind: kind.clone(),
                value: value.clone(),
            },
            InstanceError::InvalidGameVersion { value, reason } => Self::InvalidGameVersion {
                value: value.clone(),
                reason: (*reason).into(),
            },
            InstanceError::InvalidPagination {
                field,
                value,
                reason,
            } => Self::InvalidPagination {
                field: (*field).into(),
                value: value.to_string(),
                reason: (*reason).into(),
            },
            InstanceError::InvalidLoaderVersion { value, reason } => Self::InvalidLoaderVersion {
                value: value.clone(),
                reason: (*reason).into(),
            },
            InstanceError::PathEscapesInstance { path, instance_id } => Self::PathEscapesInstance {
                path: path.clone(),
                instance_id: instance_id.clone(),
            },
            InstanceError::FileTooLarge { path, size, limit } => Self::FileTooLarge {
                path: path.clone(),
                size: size.to_string(),
                limit: limit.to_string(),
            },
            InstanceError::ContentDuplication { content_path } => Self::ContentDuplication {
                content_path: content_path.clone(),
            },
            InstanceError::ContentFilename { path } => Self::ContentFilename {
                path: path.to_string_lossy().to_string(),
            },
            InstanceError::ContentDownloadError(err) => Self::ContentDownloadError(err.clone()),
            InstanceError::ContentProviderError { reason } => Self::ContentProviderError {
                reason: reason.clone(),
            },
            InstanceError::ContentForGameVersionNotFound { game_version } => {
                Self::ContentForGameVersionNotFound {
                    game_version: game_version.clone(),
                }
            }
            InstanceError::UnsupportedContentType { content_type } => {
                Self::UnsupportedContentType {
                    content_type: (*content_type).into(),
                }
            }
            InstanceError::UnsupportedOperation {
                provider_id,
                operation,
            } => Self::UnsupportedOperation {
                provider_id: provider_id.to_string(),
                operation: (*operation).to_owned(),
            },
            InstanceError::PlatformProviderNotFound { platform } => {
                Self::PlatformProviderNotFound {
                    platform: platform.to_string(),
                }
            }
            InstanceError::PlatformProviderNotChosen { platform } => {
                Self::PlatformProviderNotChosen {
                    platform: platform.to_string(),
                }
            }
            InstanceError::GameChangeNotAllowed {
                instance_id,
                provider_id,
                content_id,
            } => Self::GameChangeNotAllowed {
                instance_id: instance_id.clone(),
                provider_id: provider_id.to_string(),
                content_id: content_id.clone(),
            },
            InstanceError::SettingsLoadError(err) => {
                return Err(Box::new(FrontendError::Settings(err.into())));
            }
            InstanceError::MinecraftError(err) => {
                return Err(Box::new(FrontendError::Minecraft(err.into())));
            }
            InstanceError::ProcessError(err) => {
                return Err(Box::new(FrontendError::Process(err.into())));
            }
            InstanceError::CredentialsError(err) => {
                return Err(Box::new(FrontendError::Auth(err.into())));
            }
        })
    }
}

/// Why a path is not a path inside an instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PathRejectionDto {
    Empty,
    NulByte,
    Backslash,
    Absolute,
    EmptySegment,
    DotSegment,
    Colon,
    TrailingDotOrSpace,
    ReservedName,
    NotPlain,
    EscapesRoot,
    NotUtf8,
    NotSingleSegment,
    NotRegularFile,
    LauncherFolder,
}

impl From<PathRejection> for PathRejectionDto {
    fn from(value: PathRejection) -> Self {
        match value {
            PathRejection::Empty => Self::Empty,
            PathRejection::NulByte => Self::NulByte,
            PathRejection::Backslash => Self::Backslash,
            PathRejection::Absolute => Self::Absolute,
            PathRejection::EmptySegment => Self::EmptySegment,
            PathRejection::DotSegment => Self::DotSegment,
            PathRejection::Colon => Self::Colon,
            PathRejection::TrailingDotOrSpace => Self::TrailingDotOrSpace,
            PathRejection::ReservedName => Self::ReservedName,
            PathRejection::NotPlain => Self::NotPlain,
            PathRejection::EscapesRoot => Self::EscapesRoot,
            PathRejection::NotUtf8 => Self::NotUtf8,
            PathRejection::NotSingleSegment => Self::NotSingleSegment,
            PathRejection::NotRegularFile => Self::NotRegularFile,
            PathRejection::LauncherFolder => Self::LauncherFolder,
        }
    }
}

/// Why a text is not a Minecraft version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameVersionReasonDto {
    Empty,
    TooLong,
    SurroundingWhitespace,
    ControlCharacter,
    ForbiddenCharacter { char: String },
    Dots,
    ReservedName,
}

impl From<GameVersionRejection> for GameVersionReasonDto {
    fn from(value: GameVersionRejection) -> Self {
        match value {
            GameVersionRejection::Empty => Self::Empty,
            GameVersionRejection::TooLong => Self::TooLong,
            GameVersionRejection::SurroundingWhitespace => Self::SurroundingWhitespace,
            GameVersionRejection::ControlCharacter => Self::ControlCharacter,
            GameVersionRejection::ForbiddenCharacter(char) => Self::ForbiddenCharacter {
                char: char.to_string(),
            },
            GameVersionRejection::Dots => Self::Dots,
            GameVersionRejection::ReservedName => Self::ReservedName,
        }
    }
}

/// Why a text is not a loader version id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoaderVersionReasonDto {
    Empty,
    TooLong,
    ControlCharacter,
    Whitespace,
}

impl From<LoaderVersionIdRejection> for LoaderVersionReasonDto {
    fn from(value: LoaderVersionIdRejection) -> Self {
        match value {
            LoaderVersionIdRejection::Empty => Self::Empty,
            LoaderVersionIdRejection::TooLong => Self::TooLong,
            LoaderVersionIdRejection::ControlCharacter => Self::ControlCharacter,
            LoaderVersionIdRejection::Whitespace => Self::Whitespace,
        }
    }
}

/// Which pagination value was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaginationFieldDto {
    Page,
    PageSize,
}

impl From<PaginationField> for PaginationFieldDto {
    fn from(value: PaginationField) -> Self {
        match value {
            PaginationField::Page => Self::Page,
            PaginationField::PageSize => Self::PageSize,
        }
    }
}

/// Why a number is not a page index or a page size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaginationReasonDto {
    TooSmall,
    TooLarge { max: u32 },
}

impl From<PaginationRejection> for PaginationReasonDto {
    fn from(value: PaginationRejection) -> Self {
        match value {
            PaginationRejection::TooSmall => Self::TooSmall,
            PaginationRejection::TooLarge(max) => Self::TooLarge { max },
        }
    }
}

/// Why a text is not a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandParseReasonDto {
    Empty,
}

impl From<CommandParseError> for CommandParseReasonDto {
    fn from(value: CommandParseError) -> Self {
        match value {
            CommandParseError::Empty => Self::Empty,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum InstanceFieldDto {
    Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum InstanceValidationErrorReasonDto {
    CannotBeEmpty,
}

impl From<InstanceField> for InstanceFieldDto {
    fn from(value: InstanceField) -> Self {
        match value {
            InstanceField::Name => Self::Name,
        }
    }
}

impl From<InstanceValidationErrorReason> for InstanceValidationErrorReasonDto {
    fn from(value: InstanceValidationErrorReason) -> Self {
        match value {
            InstanceValidationErrorReason::CannotBeEmpty => Self::CannotBeEmpty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::features::auth::{AuthApplicationError, AuthDomainError};
    use aether_core::features::minecraft::MinecraftApplicationError;
    use aether_core::features::process::ProcessError;
    use aether_core::features::settings::SettingsError;
    use aether_core::shared::io::IoError;
    use serde_json::{Value, json};

    fn to_json(err: &InstanceError) -> Value {
        match FrontendError::from(err) {
            FrontendError::Instance(dto) => serde_json::to_value(dto).unwrap(),
            other => panic!("not an instance error: {other:?}"),
        }
    }

    #[test]
    fn should_report_a_wrapped_settings_error_as_a_settings_error() {
        let err = InstanceError::SettingsLoadError(SettingsError::NotFound);

        assert!(matches!(
            FrontendError::from(&err),
            FrontendError::Settings(_)
        ));
    }

    #[test]
    fn should_report_a_wrapped_minecraft_error_as_a_minecraft_error() {
        let err = InstanceError::MinecraftError(MinecraftApplicationError::Storage(
            IoError::IoError(std::io::Error::from(std::io::ErrorKind::NotFound)),
        ));

        assert!(matches!(
            FrontendError::from(&err),
            FrontendError::Minecraft(_)
        ));
    }

    #[test]
    fn should_report_a_wrapped_process_error_as_a_process_error() {
        let err = InstanceError::ProcessError(ProcessError::KillError { id: "p".into() });

        assert!(matches!(
            FrontendError::from(&err),
            FrontendError::Process(_)
        ));
    }

    #[test]
    fn should_report_a_wrapped_credentials_error_as_an_auth_error() {
        let err = InstanceError::CredentialsError(AuthApplicationError::Domain(
            AuthDomainError::NoActiveCredentials,
        ));

        assert!(matches!(FrontendError::from(&err), FrontendError::Auth(_)));
    }

    #[test]
    fn should_keep_an_instance_error_as_an_instance_error() {
        let err = InstanceError::NotFound {
            instance_id: "i".into(),
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "NOT_FOUND", "payload": {"instance_id": "i"}})
        );
    }

    #[test]
    fn should_serialize_storage_with_an_io_cause() {
        let err = InstanceError::storage(std::io::Error::from(std::io::ErrorKind::NotFound));

        assert_eq!(
            to_json(&err),
            json!({"code": "STORAGE", "payload": {"cause": {"type": "io", "kind": "notFound"}}})
        );
    }

    #[test]
    fn should_serialize_storage_with_a_database_cause() {
        let err = InstanceError::storage(sqlx::Error::RowNotFound);

        assert_eq!(
            to_json(&err),
            json!({"code": "STORAGE", "payload": {"cause": {"type": "database"}}})
        );
    }

    #[test]
    fn should_serialize_storage_without_a_category_as_unknown() {
        let err = InstanceError::storage(StorageError::other("odd"));

        assert_eq!(
            to_json(&err),
            json!({"code": "STORAGE", "payload": {"cause": {"type": "unknown", "details": "odd"}}})
        );
    }

    #[test]
    fn should_serialize_an_empty_prelaunch_command_with_a_reason() {
        let err = InstanceError::PrelaunchCommandInvalid {
            reason: CommandParseError::Empty,
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "PRELAUNCH_COMMAND_INVALID", "payload": {"reason": {"code": "EMPTY"}}})
        );
    }

    #[test]
    fn should_serialize_a_prelaunch_command_that_cannot_start_with_an_io_cause() {
        let err = InstanceError::PrelaunchCommandSpawnFailed {
            program: "x".into(),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "PRELAUNCH_COMMAND_SPAWN_FAILED", "payload": {"cause": {"type": "io", "kind": "permissionDenied"}}})
        );
    }

    #[test]
    fn should_serialize_an_invalid_relative_path_with_a_reason() {
        let err = InstanceError::InvalidRelativePath {
            path: "a/../b".into(),
            reason: PathRejection::DotSegment,
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "INVALID_RELATIVE_PATH", "payload": {"path": "a/../b", "reason": {"code": "DOT_SEGMENT"}}})
        );
    }

    #[test]
    fn should_serialize_an_invalid_game_version_with_the_forbidden_character() {
        let err = InstanceError::InvalidGameVersion {
            value: "1/2".into(),
            reason: GameVersionRejection::ForbiddenCharacter('/'),
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "INVALID_GAME_VERSION", "payload": {"value": "1/2", "reason": {"code": "FORBIDDEN_CHARACTER", "payload": {"char": "/"}}}})
        );
    }

    #[test]
    fn should_serialize_an_invalid_loader_version_with_a_reason() {
        let err = InstanceError::InvalidLoaderVersion {
            value: "1 2".into(),
            reason: LoaderVersionIdRejection::Whitespace,
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "INVALID_LOADER_VERSION", "payload": {"value": "1 2", "reason": {"code": "WHITESPACE"}}})
        );
    }

    #[test]
    fn should_serialize_an_invalid_pagination_with_the_limit() {
        let err = InstanceError::InvalidPagination {
            field: PaginationField::PageSize,
            value: 5000,
            reason: PaginationRejection::TooLarge(1000),
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "INVALID_PAGINATION", "payload": {"field": "PAGE_SIZE", "value": "5000", "reason": {"code": "TOO_LARGE", "payload": {"max": 1000}}}})
        );
    }

    #[test]
    fn should_serialize_a_failed_import_with_the_text_of_the_plugin_as_unknown() {
        let err = InstanceError::ImportFailed {
            plugin_id: "p".into(),
            capability_id: "c".into(),
            reason: "boom".into(),
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "IMPORT_FAILED", "payload": {"plugin_id": "p", "capability_id": "c", "cause": {"type": "unknown", "details": "boom"}}})
        );
    }
}
