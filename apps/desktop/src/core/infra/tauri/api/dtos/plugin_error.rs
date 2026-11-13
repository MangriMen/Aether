use aether_core::features::plugins::{
    ManifestError, ManifestFormatError, PluginCallError, PluginError, PluginLoadFailure,
    PluginProviderError, ProviderFailure,
};
use serde::Serialize;
use specta::Type;

use super::{CauseDto, FrontendError, without_query};
use crate::features::plugins::{
    AllowedHostEntryRejectionDto, AllowedHostRejectionDto, AllowedPathRejectionDto,
    LoadConfigTypeDto,
};

/// Why a plugin manifest is refused.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManifestErrorDto {
    UnsupportedApi,
    InvalidSemver {
        field: String,
        value: String,
        cause: CauseDto,
    },
    InvalidPathMapping,
    InvalidLoadFile {
        path: String,
    },
    UnknownHostFunctions {
        names: Vec<String>,
    },
    HostFunctionsNotDeclared,
    InvalidId {
        field: String,
        value: String,
    },
    ReservedPluginId,
    PlatformWithoutResolveFile {
        capability_id: String,
    },
}

impl From<&ManifestError> for ManifestErrorDto {
    fn from(value: &ManifestError) -> Self {
        match value {
            ManifestError::UnsupportedApi => Self::UnsupportedApi,
            // The semver parser's own text is shown only among the details.
            ManifestError::InvalidSemver {
                field,
                value,
                error,
            } => Self::InvalidSemver {
                field: field.clone(),
                value: value.clone(),
                cause: CauseDto::unknown(error),
            },
            ManifestError::InvalidPathMapping => Self::InvalidPathMapping,
            ManifestError::InvalidLoadFile(path) => Self::InvalidLoadFile { path: path.clone() },
            ManifestError::UnknownHostFunctions(names) => Self::UnknownHostFunctions {
                names: names.clone(),
            },
            ManifestError::HostFunctionsNotDeclared => Self::HostFunctionsNotDeclared,
            ManifestError::InvalidId { field, value } => Self::InvalidId {
                field: field.clone(),
                value: value.clone(),
            },
            ManifestError::ReservedPluginId => Self::ReservedPluginId,
            ManifestError::PlatformWithoutResolveFile { capability_id } => {
                Self::PlatformWithoutResolveFile {
                    capability_id: capability_id.to_string(),
                }
            }
        }
    }
}

/// Why the loader could not bring a plugin up.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PluginLoadFailureDto {
    HashMismatch {
        path: String,
        actual_sha256: String,
        recorded_sha256: String,
        source_type: String,
    },
    Build {
        cause: CauseDto,
    },
    ContainerUnavailable,
    Other {
        cause: CauseDto,
    },
}

impl From<&PluginLoadFailure> for PluginLoadFailureDto {
    fn from(value: &PluginLoadFailure) -> Self {
        match value {
            PluginLoadFailure::HashMismatch {
                path,
                actual_sha256,
                recorded_sha256,
                source_type,
            } => Self::HashMismatch {
                path: path.clone(),
                actual_sha256: actual_sha256.clone(),
                recorded_sha256: recorded_sha256.clone(),
                source_type: source_type.to_string(),
            },
            // The runtime's own text: not ours to categorize.
            PluginLoadFailure::Build(details) => Self::Build {
                cause: CauseDto::unknown(details),
            },
            PluginLoadFailure::ContainerUnavailable => Self::ContainerUnavailable,
            PluginLoadFailure::Other(details) => Self::Other {
                cause: CauseDto::unknown(details),
            },
        }
    }
}

/// Why a call into a plugin failed.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PluginCallReasonDto {
    MissingFunction,
    Timeout,
    OutOfFuel,
    OutOfMemory,
    Trap { cause: CauseDto },
    NonZeroExit { code: i32 },
    Plugin { cause: CauseDto },
    Codec { cause: CauseDto },
    Reentrant,
}

impl From<&PluginCallError> for PluginCallReasonDto {
    fn from(value: &PluginCallError) -> Self {
        match value {
            PluginCallError::MissingFunction => Self::MissingFunction,
            PluginCallError::Timeout => Self::Timeout,
            PluginCallError::OutOfFuel => Self::OutOfFuel,
            PluginCallError::OutOfMemory => Self::OutOfMemory,
            // The texts of the runtime and of the plugin itself: not ours to categorize.
            PluginCallError::Trap(details) => Self::Trap {
                cause: CauseDto::unknown(details),
            },
            PluginCallError::NonZeroExit(code) => Self::NonZeroExit { code: *code },
            PluginCallError::Plugin(details) => Self::Plugin {
                cause: CauseDto::unknown(details),
            },
            PluginCallError::Codec(details) => Self::Codec {
                cause: CauseDto::unknown(details),
            },
            PluginCallError::Reentrant => Self::Reentrant,
        }
    }
}

/// What went wrong talking to a plugin source.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderFailureDto {
    Request {
        cause: CauseDto,
    },
    TooLarge {
        // Formatted before it crosses the boundary: no `u64` in bindings.
        limit: String,
    },
    UrlNotAllowed,
    NoReleases,
    InvalidUrl {
        url: String,
    },
    NotRegistered,
}

impl From<&ProviderFailure> for ProviderFailureDto {
    fn from(value: &ProviderFailure) -> Self {
        match value {
            ProviderFailure::Request(source) => Self::Request {
                cause: CauseDto::of_source(source.as_ref()),
            },
            ProviderFailure::Status { status, url } => Self::Request {
                cause: CauseDto::HttpStatus {
                    status: *status,
                    url: without_query(url),
                },
            },
            ProviderFailure::TooLarge { limit } => Self::TooLarge {
                limit: limit.to_string(),
            },
            ProviderFailure::UrlNotAllowed => Self::UrlNotAllowed,
            ProviderFailure::NoReleases => Self::NoReleases,
            ProviderFailure::InvalidUrl { url } => Self::InvalidUrl {
                url: without_query(url),
            },
            ProviderFailure::NotRegistered => Self::NotRegistered,
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PluginErrorDto {
    NotFound {
        plugin_id: String,
    },
    AlreadyLoaded {
        plugin_id: String,
    },
    LoadingInProgress {
        plugin_id: String,
    },
    UnloadingInProgress {
        plugin_id: String,
    },
    AlreadyUnloaded {
        plugin_id: String,
    },
    Busy {
        plugin_id: String,
    },
    Stopped {
        plugin_id: String,
    },
    IdMismatch {
        plugin_id: String,
        archive_id: String,
    },
    LoadFailed {
        plugin_id: String,
        reason: PluginLoadFailureDto,
    },
    IncompatibleApiVersion {
        plugin_id: String,
        required: String,
        host: String,
    },
    FunctionCallFailed {
        function_name: String,
        plugin_id: String,
        reason: PluginCallReasonDto,
    },
    ManifestNotFound {
        path: String,
    },
    InvalidManifestFormat {
        cause: CauseDto,
    },
    ManifestTooLarge {
        name: String,
        // Formatted before it crosses the boundary: no `u64` in bindings.
        limit: String,
    },
    Manifest {
        error: ManifestErrorDto,
    },
    LoaderNotFound {
        config_type: LoadConfigTypeDto,
    },
    ExtractionFailed {
        from: String,
    },
    InvalidExtractionFormat,
    FileExtractionFailed {
        from: String,
    },
    CapabilityRegistrationFailed {
        capability_type: String,
        capability_id: String,
    },
    CapabilityCancelRegistrationFailed {
        capability_type: String,
        capability_id: String,
    },
    AccessViolation {
        plugin_id: String,
        path: String,
    },
    InvalidAllowedPath {
        plugin_id: String,
        path: String,
        reason: AllowedPathRejectionDto,
    },
    InvalidAllowedHost {
        plugin_id: String,
        url: String,
        reason: AllowedHostRejectionDto,
    },
    InvalidAllowedHostEntry {
        plugin_id: String,
        entry: String,
        reason: AllowedHostEntryRejectionDto,
    },
    HashComputationFailed,
    Storage {
        cause: CauseDto,
    },
    // ── Provider errors (provider-agnostic) ──
    ProviderFetchError {
        source_type: String,
        reason: ProviderFailureDto,
    },
    ProviderNoAssets {
        source_type: String,
    },
    ArchiveChangedSincePreview,
    NoRemoteSource {
        plugin_id: String,
    },
    DownloadFailed {
        url: String,
        reason: ProviderFailureDto,
    },
    // ── Rate limiting ──
    ProviderRateLimited {
        source_type: String,
        retry_after: Option<u32>,
    },
}

impl From<&PluginError> for FrontendError {
    /// A settings error wrapped by the plugin one is reported as its own: the frontend
    /// translates it by what failed, not by who was asking.
    fn from(value: &PluginError) -> Self {
        match PluginErrorDto::own(value) {
            Ok(dto) => Self::Plugin(dto),
            Err(lifted) => *lifted,
        }
    }
}

impl PluginErrorDto {
    /// The DTO of an error the plugin feature raises itself; an error of another domain it
    /// wraps comes back as that domain's own `FrontendError`.
    // One flat arm per variant; splitting an exhaustive mapping only hides which ones are covered.
    #[allow(clippy::too_many_lines)]
    fn own(value: &PluginError) -> Result<Self, Box<FrontendError>> {
        Ok(match value {
            PluginError::NotFound { plugin_id } => Self::NotFound {
                plugin_id: plugin_id.clone(),
            },
            PluginError::AlreadyLoaded { plugin_id } => Self::AlreadyLoaded {
                plugin_id: plugin_id.to_string(),
            },
            PluginError::LoadingInProgress { plugin_id } => Self::LoadingInProgress {
                plugin_id: plugin_id.to_string(),
            },
            PluginError::UnloadingInProgress { plugin_id } => Self::UnloadingInProgress {
                plugin_id: plugin_id.to_string(),
            },
            PluginError::AlreadyUnloaded { plugin_id } => Self::AlreadyUnloaded {
                plugin_id: plugin_id.to_string(),
            },
            PluginError::Busy { plugin_id } => Self::Busy {
                plugin_id: plugin_id.to_string(),
            },
            PluginError::Stopped { plugin_id } => Self::Stopped {
                plugin_id: plugin_id.to_string(),
            },
            PluginError::IdMismatch {
                plugin_id,
                archive_id,
            } => Self::IdMismatch {
                plugin_id: plugin_id.to_string(),
                archive_id: archive_id.to_string(),
            },
            PluginError::LoadFailed { plugin_id, reason } => Self::LoadFailed {
                plugin_id: plugin_id.to_string(),
                reason: reason.into(),
            },
            PluginError::IncompatibleApiVersion {
                plugin_id,
                required,
                host,
            } => Self::IncompatibleApiVersion {
                plugin_id: plugin_id.to_string(),
                required: required.to_string(),
                host: host.to_string(),
            },
            PluginError::FunctionCallFailed {
                function_name,
                plugin_id,
                error,
            } => Self::FunctionCallFailed {
                function_name: function_name.clone(),
                plugin_id: plugin_id.to_string(),
                reason: error.into(),
            },
            PluginError::Manifest(err) => Self::Manifest { error: err.into() },
            PluginError::LoaderNotFound { config_type } => Self::LoaderNotFound {
                config_type: (*config_type).into(),
            },
            PluginError::CapabilityRegistrationFailed {
                capability_type,
                capability_id,
            } => Self::CapabilityRegistrationFailed {
                capability_type: capability_type.to_string(),
                capability_id: capability_id.to_string(),
            },
            PluginError::CapabilityCancelRegistrationFailed {
                capability_type,
                capability_id,
            } => Self::CapabilityCancelRegistrationFailed {
                capability_type: capability_type.to_string(),
                capability_id: capability_id.to_string(),
            },
            PluginError::AccessViolation { plugin_id, path } => Self::AccessViolation {
                plugin_id: plugin_id.to_string(),
                path: path.clone(),
            },
            PluginError::InvalidAllowedPath {
                plugin_id,
                path,
                reason,
            } => Self::InvalidAllowedPath {
                plugin_id: plugin_id.to_string(),
                path: path.clone(),
                reason: (*reason).into(),
            },
            PluginError::InvalidAllowedHost {
                plugin_id,
                url,
                reason,
            } => Self::InvalidAllowedHost {
                plugin_id: plugin_id.to_string(),
                url: url.clone(),
                reason: (*reason).into(),
            },
            PluginError::InvalidAllowedHostEntry {
                plugin_id,
                entry,
                reason,
            } => Self::InvalidAllowedHostEntry {
                plugin_id: plugin_id.to_string(),
                entry: entry.clone(),
                reason: (*reason).into(),
            },
            PluginError::HashComputationFailed => Self::HashComputationFailed,
            PluginError::Storage(err) => Self::Storage { cause: err.into() },
            PluginError::Provider(err) => err.into(),
            PluginError::Settings(err) => {
                return Err(Box::new(FrontendError::Settings(err.into())));
            }
        })
    }
}

impl From<&PluginProviderError> for PluginErrorDto {
    fn from(value: &PluginProviderError) -> Self {
        match value {
            PluginProviderError::ManifestNotFound { path } => {
                Self::ManifestNotFound { path: path.clone() }
            }
            PluginProviderError::InvalidManifestFormat {
                error: ManifestFormatError::TooLarge { name, limit },
            } => Self::ManifestTooLarge {
                name: name.clone(),
                limit: limit.to_string(),
            },
            PluginProviderError::InvalidManifestFormat {
                error: ManifestFormatError::Json(err),
            } => Self::InvalidManifestFormat { cause: err.into() },
            PluginProviderError::InvalidManifestFormat {
                error: ManifestFormatError::Utf8(_),
            } => Self::InvalidManifestFormat {
                cause: CauseDto::Parse {
                    line: None,
                    column: None,
                },
            },
            PluginProviderError::ExtractionFailed { from } => {
                Self::ExtractionFailed { from: from.clone() }
            }
            PluginProviderError::InvalidExtractionFormat => Self::InvalidExtractionFormat,
            PluginProviderError::FileExtractionFailed { from } => {
                Self::FileExtractionFailed { from: from.clone() }
            }
            PluginProviderError::ProviderFetchError {
                source_type,
                reason,
            } => Self::ProviderFetchError {
                source_type: source_type.to_string(),
                reason: reason.into(),
            },
            PluginProviderError::ProviderNoAssets { source_type } => Self::ProviderNoAssets {
                source_type: source_type.to_string(),
            },
            PluginProviderError::ArchiveChangedSincePreview => Self::ArchiveChangedSincePreview,
            PluginProviderError::NoRemoteSource { plugin_id } => Self::NoRemoteSource {
                plugin_id: plugin_id.to_string(),
            },
            PluginProviderError::DownloadFailed { url, reason } => Self::DownloadFailed {
                url: without_query(url),
                reason: reason.into(),
            },
            PluginProviderError::ProviderRateLimited {
                source_type,
                retry_after,
            } => Self::ProviderRateLimited {
                source_type: source_type.to_string(),
                retry_after: *retry_after,
            },
            PluginProviderError::Manifest(err) => Self::Manifest { error: err.into() },
            PluginProviderError::Storage(err) => Self::Storage { cause: err.into() },
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_core::features::{
        instance::{CapabilityId, PluginId},
        plugins::{PluginCallError, PluginSourceType},
        settings::SettingsError,
    };
    use aether_core::shared::io::IoError;
    use serde_json::json;

    use super::*;

    fn frontend(err: &PluginError) -> serde_json::Value {
        serde_json::to_value(FrontendError::from(err)).unwrap()
    }

    fn plugin(err: &PluginError) -> serde_json::Value {
        frontend(err)["payload"].clone()
    }

    fn manifest(err: &ManifestError) -> serde_json::Value {
        serde_json::to_value(ManifestErrorDto::from(err)).unwrap()
    }

    #[test]
    fn should_serialize_every_manifest_error_as_its_code_and_arguments() {
        assert_eq!(
            manifest(&ManifestError::UnsupportedApi),
            json!({"code": "UNSUPPORTED_API"})
        );
        assert_eq!(
            manifest(&ManifestError::InvalidSemver {
                field: "version".to_owned(),
                value: "1.x".to_owned(),
                error: "unexpected character".to_owned(),
            }),
            json!({"code": "INVALID_SEMVER", "payload": {
                "field": "version",
                "value": "1.x",
                "cause": {"type": "unknown", "details": "unexpected character"},
            }})
        );
        assert_eq!(
            manifest(&ManifestError::InvalidPathMapping),
            json!({"code": "INVALID_PATH_MAPPING"})
        );
        assert_eq!(
            manifest(&ManifestError::InvalidLoadFile("../a.wasm".to_owned())),
            json!({"code": "INVALID_LOAD_FILE", "payload": {"path": "../a.wasm"}})
        );
        assert_eq!(
            manifest(&ManifestError::UnknownHostFunctions(vec![
                "a".to_owned(),
                "b".to_owned()
            ])),
            json!({"code": "UNKNOWN_HOST_FUNCTIONS", "payload": {"names": ["a", "b"]}})
        );
        assert_eq!(
            manifest(&ManifestError::HostFunctionsNotDeclared),
            json!({"code": "HOST_FUNCTIONS_NOT_DECLARED"})
        );
        assert_eq!(
            manifest(&ManifestError::InvalidId {
                field: "id".to_owned(),
                value: "Bad".to_owned(),
            }),
            json!({"code": "INVALID_ID", "payload": {"field": "id", "value": "Bad"}})
        );
        assert_eq!(
            manifest(&ManifestError::ReservedPluginId),
            json!({"code": "RESERVED_PLUGIN_ID"})
        );
        assert_eq!(
            manifest(&ManifestError::PlatformWithoutResolveFile {
                capability_id: CapabilityId::from_static("modrinth"),
            }),
            json!({"code": "PLATFORM_WITHOUT_RESOLVE_FILE", "payload": {"capability_id": "modrinth"}})
        );
    }

    #[test]
    fn should_serialize_a_manifest_error_inside_the_plugin_error() {
        let json = plugin(&PluginError::Manifest(ManifestError::ReservedPluginId));

        assert_eq!(
            json,
            json!({"code": "MANIFEST", "payload": {"error": {"code": "RESERVED_PLUGIN_ID"}}})
        );
    }

    #[test]
    fn should_serialize_every_load_failure_as_its_code_and_arguments() {
        let failed = |reason| PluginError::LoadFailed {
            plugin_id: PluginId::from_static("p"),
            reason,
        };

        assert_eq!(
            plugin(&failed(PluginLoadFailure::HashMismatch {
                path: "p/a.wasm".to_owned(),
                actual_sha256: "aa".to_owned(),
                recorded_sha256: "bb".to_owned(),
                source_type: PluginSourceType::GitHub,
            })),
            json!({"code": "LOAD_FAILED", "payload": {"plugin_id": "p", "reason": {
                "code": "HASH_MISMATCH",
                "payload": {
                    "path": "p/a.wasm",
                    "actual_sha256": "aa",
                    "recorded_sha256": "bb",
                    "source_type": "github",
                },
            }}})
        );
        assert_eq!(
            plugin(&failed(PluginLoadFailure::Build("bad import".to_owned()))),
            json!({"code": "LOAD_FAILED", "payload": {"plugin_id": "p", "reason": {
                "code": "BUILD",
                "payload": {"cause": {"type": "unknown", "details": "bad import"}},
            }}})
        );
        assert_eq!(
            plugin(&failed(PluginLoadFailure::ContainerUnavailable)),
            json!({"code": "LOAD_FAILED", "payload": {"plugin_id": "p", "reason": {
                "code": "CONTAINER_UNAVAILABLE",
            }}})
        );
        assert_eq!(
            plugin(&failed(PluginLoadFailure::Other("storage down".to_owned()))),
            json!({"code": "LOAD_FAILED", "payload": {"plugin_id": "p", "reason": {
                "code": "OTHER",
                "payload": {"cause": {"type": "unknown", "details": "storage down"}},
            }}})
        );
    }

    #[test]
    fn should_serialize_the_versions_of_an_incompatible_plugin_as_text() {
        let err = PluginError::IncompatibleApiVersion {
            plugin_id: PluginId::from_static("p"),
            required: semver::VersionReq::parse("^2.0.0").unwrap(),
            host: semver::Version::new(1, 4, 0),
        };

        assert_eq!(
            plugin(&err),
            json!({"code": "INCOMPATIBLE_API_VERSION", "payload": {
                "plugin_id": "p", "required": "^2.0.0", "host": "1.4.0",
            }})
        );
    }

    fn call_failed(error: PluginCallError) -> serde_json::Value {
        plugin(&PluginError::FunctionCallFailed {
            function_name: "list".to_owned(),
            plugin_id: PluginId::from_static("p"),
            error,
        })
    }

    #[test]
    fn should_keep_the_text_of_a_failed_plugin_call_as_an_unknown_cause_of_its_reason() {
        assert_eq!(
            call_failed(PluginCallError::Plugin("denied".to_owned())),
            json!({"code": "FUNCTION_CALL_FAILED", "payload": {
                "function_name": "list",
                "plugin_id": "p",
                "reason": {"code": "PLUGIN", "payload": {"cause": {"type": "unknown", "details": "denied"}}},
            }})
        );
    }

    #[test]
    fn should_name_the_limit_a_plugin_call_hit_as_its_reason() {
        for (error, code) in [
            (PluginCallError::MissingFunction, "MISSING_FUNCTION"),
            (PluginCallError::Timeout, "TIMEOUT"),
            (PluginCallError::OutOfFuel, "OUT_OF_FUEL"),
            (PluginCallError::OutOfMemory, "OUT_OF_MEMORY"),
            (PluginCallError::Reentrant, "REENTRANT"),
        ] {
            assert_eq!(
                call_failed(error)["payload"]["reason"],
                json!({"code": code})
            );
        }
        assert_eq!(
            call_failed(PluginCallError::NonZeroExit(3))["payload"]["reason"],
            json!({"code": "NON_ZERO_EXIT", "payload": {"code": 3}})
        );
        assert_eq!(
            call_failed(PluginCallError::Trap("unreachable".to_owned()))["payload"]["reason"]["code"],
            "TRAP"
        );
    }

    #[test]
    fn should_report_a_settings_error_as_a_settings_error_when_a_plugin_error_wraps_it() {
        let json = frontend(&PluginError::Settings(SettingsError::InvalidMemory));

        assert_eq!(
            json,
            json!({"type": "settings", "payload": {"code": "INVALID_MEMORY"}})
        );
    }

    #[test]
    fn should_categorize_the_io_error_of_plugin_storage() {
        let err = PluginError::Storage(IoError::from(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )));

        assert_eq!(
            plugin(&err),
            json!({"code": "STORAGE", "payload": {"cause": {"type": "io", "kind": "notFound"}}})
        );
    }

    #[test]
    fn should_categorize_the_typed_source_of_a_provider_failure() {
        let fetch = |reason| {
            PluginError::Provider(PluginProviderError::ProviderFetchError {
                source_type: PluginSourceType::GitHub,
                reason,
            })
        };

        assert_eq!(
            plugin(&fetch(ProviderFailure::Status {
                status: 403,
                url: "https://api.github.com/repos/o/r/releases?per_page=20".to_owned(),
            })),
            json!({"code": "PROVIDER_FETCH_ERROR", "payload": {
                "source_type": "github",
                "reason": {"code": "REQUEST", "payload": {"cause": {
                    "type": "httpStatus",
                    "status": 403,
                    "url": "https://api.github.com/repos/o/r/releases",
                }}},
            }})
        );
        assert_eq!(
            plugin(&fetch(ProviderFailure::Request(std::sync::Arc::new(
                std::io::Error::from(std::io::ErrorKind::TimedOut)
            ))))["payload"]["reason"],
            json!({"code": "REQUEST", "payload": {"cause": {"type": "io", "kind": "timedOut"}}})
        );
    }

    #[test]
    fn should_serialize_each_own_reason_of_a_provider_failure_with_its_arguments() {
        let fetch = |reason| {
            plugin(&PluginError::Provider(
                PluginProviderError::ProviderFetchError {
                    source_type: PluginSourceType::GitHub,
                    reason,
                },
            ))["payload"]["reason"]
                .clone()
        };

        assert_eq!(
            fetch(ProviderFailure::NoReleases),
            json!({"code": "NO_RELEASES"})
        );
        assert_eq!(
            fetch(ProviderFailure::NotRegistered),
            json!({"code": "NOT_REGISTERED"})
        );
        assert_eq!(
            fetch(ProviderFailure::InvalidUrl {
                url: "https://example.com/x?y=1".to_owned()
            }),
            json!({"code": "INVALID_URL", "payload": {"url": "https://example.com/x"}})
        );
    }

    #[test]
    fn should_serialize_a_download_over_the_size_limit_with_the_limit_as_text() {
        let err = PluginError::Provider(PluginProviderError::DownloadFailed {
            url: "https://github.com/o/r/a.zip?token=secret".to_owned(),
            reason: ProviderFailure::TooLarge { limit: 5 },
        });

        assert_eq!(
            plugin(&err),
            json!({"code": "DOWNLOAD_FAILED", "payload": {
                "url": "https://github.com/o/r/a.zip",
                "reason": {"code": "TOO_LARGE", "payload": {"limit": "5"}},
            }})
        );
    }

    #[test]
    fn should_serialize_a_manifest_over_the_size_limit_with_its_name_and_limit() {
        let err = PluginError::Provider(PluginProviderError::InvalidManifestFormat {
            error: ManifestFormatError::TooLarge {
                name: "manifest.json".to_owned(),
                limit: 64,
            },
        });

        assert_eq!(
            plugin(&err),
            json!({"code": "MANIFEST_TOO_LARGE", "payload": {"name": "manifest.json", "limit": "64"}})
        );
    }

    #[test]
    fn should_serialize_an_invalid_manifest_format_as_a_parse_cause() {
        let parse = serde_json::from_slice::<serde_json::Value>(b"{ not json").unwrap_err();
        let err = PluginError::Provider(PluginProviderError::InvalidManifestFormat {
            error: parse.into(),
        });

        assert_eq!(
            plugin(&err),
            json!({"code": "INVALID_MANIFEST_FORMAT", "payload": {
                "cause": {"type": "parse", "line": 1, "column": 3},
            }})
        );
    }

    #[test]
    fn should_categorize_the_io_error_of_provider_storage() {
        let err = PluginError::Provider(PluginProviderError::Storage(IoError::from(
            std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        )));

        assert_eq!(
            plugin(&err),
            json!({"code": "STORAGE", "payload": {"cause": {"type": "io", "kind": "permissionDenied"}}})
        );
    }
}
