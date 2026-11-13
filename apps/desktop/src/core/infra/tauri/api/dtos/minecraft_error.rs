use aether_core::features::minecraft::{
    GamePathKind, MinecraftApplicationError, MinecraftDomainError, ProcessorFailure,
};
use serde::Serialize;
use specta::Type;

use super::{CauseDto, FrontendError};
use crate::features::minecraft::LoaderVersionPreferenceDto;

#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MinecraftErrorDto {
    VersionNotFound {
        version: String,
    },
    VersionForLoaderNotFound {
        loader_version_preference: LoaderVersionPreferenceDto,
    },
    LoaderNotFound {
        loader_version_preference: LoaderVersionPreferenceDto,
    },
    LoaderVersionRequired,
    DefaultLoaderNotFound,
    ProcessorFailed {
        reason: ProcessorFailureDto,
    },
    PreLaunchFailed {
        code: i32,
    },
    PostLaunchFailed {
        code: i32,
    },
    PathNotFound {
        path: String,
        entity_type: GamePathKindDto,
    },
    ParseFailed {
        cause: CauseDto,
    },
    StorageFailure {
        cause: CauseDto,
    },
    InvalidLogConfigName {
        name: String,
    },
    Storage {
        cause: CauseDto,
    },
}

/// What a path the launch needs is the path of.
#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GamePathKindDto {
    Game,
    Assets,
    Library,
    NativeLibrary,
    LogConfig,
    Classpath,
}

impl From<GamePathKind> for GamePathKindDto {
    fn from(value: GamePathKind) -> Self {
        match value {
            GamePathKind::Game => Self::Game,
            GamePathKind::Assets => Self::Assets,
            GamePathKind::Library => Self::Library,
            GamePathKind::NativeLibrary => Self::NativeLibrary,
            GamePathKind::LogConfig => Self::LogConfig,
            GamePathKind::Classpath => Self::Classpath,
        }
    }
}

/// Why a modloader processor did not finish.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessorFailureDto {
    MainClassNotFound {
        jar: String,
    },
    UnresolvedPlaceholder {
        argument: String,
    },
    CannotReadProcessor {
        path: String,
    },
    CannotReadManifest {
        path: String,
    },
    SpawnFailed {
        cause: CauseDto,
    },
    /// `cause` carries the output of the processor: its text, not ours.
    Exited {
        code: i32,
        cause: CauseDto,
    },
    /// The OS ended the processor by a signal, so there is no exit code.
    Terminated {
        cause: CauseDto,
    },
    TaskFailed {
        cause: CauseDto,
    },
}

impl From<&ProcessorFailure> for ProcessorFailureDto {
    fn from(value: &ProcessorFailure) -> Self {
        match value {
            ProcessorFailure::MainClassNotFound { jar } => {
                Self::MainClassNotFound { jar: jar.clone() }
            }
            ProcessorFailure::UnresolvedPlaceholder { argument } => Self::UnresolvedPlaceholder {
                argument: argument.clone(),
            },
            ProcessorFailure::CannotReadProcessor { path } => {
                Self::CannotReadProcessor { path: path.clone() }
            }
            ProcessorFailure::CannotReadManifest { path } => {
                Self::CannotReadManifest { path: path.clone() }
            }
            ProcessorFailure::SpawnFailed(err) => Self::SpawnFailed { cause: err.into() },
            ProcessorFailure::Exited {
                code: Some(code),
                stderr,
            } => Self::Exited {
                code: *code,
                cause: CauseDto::unknown(stderr),
            },
            ProcessorFailure::Exited { code: None, stderr } => Self::Terminated {
                cause: CauseDto::unknown(stderr),
            },
            ProcessorFailure::TaskFailed(err) => Self::TaskFailed {
                cause: CauseDto::unknown(err),
            },
        }
    }
}

impl From<&MinecraftApplicationError> for FrontendError {
    /// A Java error the Minecraft use case passes on is reported as Java's own.
    fn from(value: &MinecraftApplicationError) -> Self {
        match value {
            MinecraftApplicationError::Domain(domain) => Self::Minecraft(domain.into()),
            MinecraftApplicationError::JavaError(err) => err.into(),
            MinecraftApplicationError::Storage(err) => {
                Self::Minecraft(MinecraftErrorDto::Storage { cause: err.into() })
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
                reason: reason.into(),
            },
            MinecraftDomainError::PreLaunchFailed { code } => Self::PreLaunchFailed { code: *code },
            MinecraftDomainError::PostLaunchFailed { code } => {
                Self::PostLaunchFailed { code: *code }
            }
            MinecraftDomainError::PathNotFound { path, entity_type } => Self::PathNotFound {
                path: path.to_string_lossy().to_string(),
                entity_type: (*entity_type).into(),
            },
            // The text of the parser (daedalus): no category to name.
            MinecraftDomainError::ParseFailed { reason } => Self::ParseFailed {
                cause: CauseDto::unknown(reason),
            },
            MinecraftDomainError::StorageFailure(err) => Self::StorageFailure { cause: err.into() },
            MinecraftDomainError::InvalidLogConfigName { name } => {
                Self::InvalidLogConfigName { name: name.clone() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::features::java::{JavaApplicationError, JavaDomainError};
    use aether_core::shared::io::IoError;
    use aether_core::shared::request_client::RequestError;
    use serde_json::{Value, json};

    fn reason(failure: ProcessorFailure) -> Value {
        let err = MinecraftDomainError::ProcessorFailed { reason: failure };
        let dto = MinecraftErrorDto::from(&err);
        serde_json::to_value(dto).unwrap()["payload"]["reason"].clone()
    }

    fn domain(err: &MinecraftDomainError) -> Value {
        serde_json::to_value(MinecraftErrorDto::from(err)).unwrap()
    }

    fn io(kind: std::io::ErrorKind) -> std::io::Error {
        std::io::Error::from(kind)
    }

    #[test]
    fn should_serialize_a_missing_path_with_what_it_is_the_path_of() {
        let out = domain(&MinecraftDomainError::PathNotFound {
            path: "libraries".into(),
            entity_type: GamePathKind::NativeLibrary,
        });

        assert_eq!(
            out,
            json!({"code": "PATH_NOT_FOUND", "payload": {"path": "libraries", "entity_type": "NATIVE_LIBRARY"}})
        );
    }

    #[test]
    fn should_serialize_a_missing_main_class_with_the_jar() {
        let out = reason(ProcessorFailure::MainClassNotFound {
            jar: "a.jar".into(),
        });

        assert_eq!(
            out,
            json!({"code": "MAIN_CLASS_NOT_FOUND", "payload": {"jar": "a.jar"}})
        );
    }

    #[test]
    fn should_serialize_an_unresolved_placeholder_with_the_argument() {
        let out = reason(ProcessorFailure::UnresolvedPlaceholder {
            argument: "x/{B}".into(),
        });

        assert_eq!(
            out,
            json!({"code": "UNRESOLVED_PLACEHOLDER", "payload": {"argument": "x/{B}"}})
        );
    }

    #[test]
    fn should_serialize_an_unreadable_processor_with_its_path() {
        let out = reason(ProcessorFailure::CannotReadProcessor {
            path: "p.jar".into(),
        });

        assert_eq!(
            out,
            json!({"code": "CANNOT_READ_PROCESSOR", "payload": {"path": "p.jar"}})
        );
    }

    #[test]
    fn should_serialize_an_unreadable_manifest_with_its_path() {
        let out = reason(ProcessorFailure::CannotReadManifest {
            path: "p.jar".into(),
        });

        assert_eq!(
            out,
            json!({"code": "CANNOT_READ_MANIFEST", "payload": {"path": "p.jar"}})
        );
    }

    #[test]
    fn should_serialize_a_processor_that_did_not_start_with_an_io_cause() {
        let out = reason(ProcessorFailure::SpawnFailed(IoError::with_path(
            io(std::io::ErrorKind::NotFound),
            "java",
        )));

        assert_eq!(
            out,
            json!({
                "code": "SPAWN_FAILED",
                "payload": {"cause": {"type": "io", "kind": "notFound"}}
            })
        );
    }

    #[test]
    fn should_serialize_an_exited_processor_with_its_code_and_output_as_unknown() {
        let out = reason(ProcessorFailure::Exited {
            code: Some(2),
            stderr: "boom".into(),
        });

        assert_eq!(
            out,
            json!({
                "code": "EXITED",
                "payload": {"code": 2, "cause": {"type": "unknown", "details": "boom"}}
            })
        );
    }

    #[test]
    fn should_serialize_a_processor_ended_by_a_signal_without_a_code() {
        let out = reason(ProcessorFailure::Exited {
            code: None,
            stderr: "killed".into(),
        });

        assert_eq!(
            out,
            json!({
                "code": "TERMINATED",
                "payload": {"cause": {"type": "unknown", "details": "killed"}}
            })
        );
    }

    #[tokio::test]
    async fn should_serialize_a_failed_processor_task_as_unknown() {
        let join = tokio::spawn(async { panic!("task blew up") })
            .await
            .unwrap_err();

        let out = reason(ProcessorFailure::TaskFailed(join.into()));

        assert_eq!(out["code"], "TASK_FAILED");
        assert_eq!(out["payload"]["cause"]["type"], "unknown");
    }

    #[test]
    fn should_serialize_a_storage_failure_with_the_category_of_its_io_source() {
        let out = domain(&MinecraftDomainError::StorageFailure(IoError::from(io(
            std::io::ErrorKind::PermissionDenied,
        ))));

        assert_eq!(
            out,
            json!({
                "code": "STORAGE_FAILURE",
                "payload": {"cause": {"type": "io", "kind": "permissionDenied"}}
            })
        );
    }

    #[test]
    fn should_serialize_a_parse_failure_with_the_text_of_the_parser_as_unknown() {
        let out = domain(&MinecraftDomainError::ParseFailed {
            reason: "bad coordinate".into(),
        });

        assert_eq!(
            out,
            json!({
                "code": "PARSE_FAILED",
                "payload": {"cause": {"type": "unknown", "details": "bad coordinate"}}
            })
        );
    }

    #[test]
    fn should_serialize_an_invalid_log_config_name_with_the_name() {
        let out = domain(&MinecraftDomainError::InvalidLogConfigName {
            name: "../x.xml".into(),
        });

        assert_eq!(
            out,
            json!({"code": "INVALID_LOG_CONFIG_NAME", "payload": {"name": "../x.xml"}})
        );
    }

    #[test]
    fn should_serialize_an_application_storage_error_with_an_io_cause() {
        let err =
            MinecraftApplicationError::Storage(IoError::from(io(std::io::ErrorKind::NotFound)));

        let FrontendError::Minecraft(dto) = FrontendError::from(&err) else {
            panic!("expected a minecraft error");
        };

        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            json!({"code": "STORAGE", "payload": {"cause": {"type": "io", "kind": "notFound"}}})
        );
    }

    #[test]
    fn should_report_a_wrapped_java_error_as_a_java_error() {
        let err = MinecraftApplicationError::JavaError(JavaApplicationError::Domain(
            JavaDomainError::NotFound { version: 17 },
        ));

        assert!(matches!(FrontendError::from(&err), FrontendError::Java(_)));
    }

    #[test]
    fn should_report_a_wrapped_java_download_failure_as_a_request_error() {
        let err = MinecraftApplicationError::JavaError(JavaApplicationError::DownloadFailed(
            RequestError::Status {
                status: 404,
                url: "https://example.com/jre".into(),
            },
        ));

        assert!(matches!(
            FrontendError::from(&err),
            FrontendError::Request(_)
        ));
    }
}
