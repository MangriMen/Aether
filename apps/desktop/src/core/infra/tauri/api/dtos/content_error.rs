use aether_core::features::instance::{
    BoxedSource, ContentDownloadFailure, ContentProviderRejection,
};
use aether_core::shared::request_client::RequestError;
use serde::Serialize;
use specta::Type;

use super::{CauseDto, without_query};
use crate::features::instance::infra::ContentTypeDto;

/// Why a content provider's answer, or the request made to it, was refused.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentProviderReasonDto {
    WrongTarget {
        provider_id: String,
    },
    ForeignPack {
        provider_id: String,
        pack_provider_id: String,
    },
    // Counts and limits cross the boundary formatted: no `usize`/`u64` in bindings.
    TooManyFiles {
        count: String,
        max: String,
    },
    TooManyOverrides {
        max: String,
    },
    InvalidOverridesArchive {
        cause: CauseDto,
    },
    VersionOfAnotherProject {
        version_id: String,
        project_id: String,
        content_id: String,
    },
    NotAModpack {
        title: String,
        content_type: ContentTypeDto,
    },
    NoGameVersion {
        version_id: String,
    },
    InvalidSearchParams {
        cause: CauseDto,
    },
    InvalidSearchResponse {
        cause: CauseDto,
    },
    UnknownProjectType {
        project_type: String,
    },
    Mapping {
        cause: CauseDto,
    },
    MrpackOpenFailed {
        cause: CauseDto,
    },
    MrpackInvalidArchive {
        cause: CauseDto,
    },
    MrpackMissingIndex,
    MrpackReadFailed {
        cause: CauseDto,
    },
    MrpackIndexTooLarge {
        max_bytes: String,
    },
    MrpackMalformedIndex {
        cause: CauseDto,
    },
    MrpackTooManyFiles {
        count: String,
        max: String,
    },
    MrpackNotMinecraft {
        game: String,
    },
    MrpackInvalidHash,
    MrpackNoAllowedDownload {
        path: String,
    },
    Provider {
        cause: CauseDto,
    },
}

/// The category of an error a provider passes on: the typed ones the core keeps, else its text.
fn source_cause(source: &BoxedSource) -> CauseDto {
    let source: &(dyn std::error::Error + 'static) = source.as_ref();
    if let Some(err) = source.downcast_ref::<RequestError>() {
        return err.into();
    }
    if let Some(err) = source.downcast_ref::<std::io::Error>() {
        return err.into();
    }
    CauseDto::unknown(source)
}

impl From<&ContentProviderRejection> for ContentProviderReasonDto {
    fn from(value: &ContentProviderRejection) -> Self {
        use ContentProviderRejection as R;

        match value {
            R::WrongTarget { provider_id } => Self::WrongTarget {
                provider_id: provider_id.to_string(),
            },
            R::ForeignPack {
                provider_id,
                pack_provider_id,
            } => Self::ForeignPack {
                provider_id: provider_id.to_string(),
                pack_provider_id: pack_provider_id.to_string(),
            },
            R::TooManyFiles { count, max } => Self::TooManyFiles {
                count: count.to_string(),
                max: max.to_string(),
            },
            R::TooManyOverrides { max } => Self::TooManyOverrides {
                max: max.to_string(),
            },
            R::InvalidOverridesArchive(err) => Self::InvalidOverridesArchive { cause: err.into() },
            R::VersionOfAnotherProject {
                version_id,
                project_id,
                content_id,
            } => Self::VersionOfAnotherProject {
                version_id: version_id.clone(),
                project_id: project_id.clone(),
                content_id: content_id.clone(),
            },
            R::NotAModpack {
                title,
                content_type,
            } => Self::NotAModpack {
                title: title.clone(),
                content_type: (*content_type).into(),
            },
            R::NoGameVersion { version_id } => Self::NoGameVersion {
                version_id: version_id.clone(),
            },
            R::InvalidSearchParams(source) => Self::InvalidSearchParams {
                cause: source_cause(source),
            },
            R::InvalidSearchResponse(source) => Self::InvalidSearchResponse {
                cause: source_cause(source),
            },
            R::UnknownProjectType { project_type } => Self::UnknownProjectType {
                project_type: project_type.clone(),
            },
            R::Mapping(source) => Self::Mapping {
                cause: source_cause(source),
            },
            R::MrpackOpenFailed(err) => Self::MrpackOpenFailed { cause: err.into() },
            R::MrpackInvalidArchive(err) => Self::MrpackInvalidArchive { cause: err.into() },
            R::MrpackMissingIndex => Self::MrpackMissingIndex,
            R::MrpackReadFailed(err) => Self::MrpackReadFailed { cause: err.into() },
            R::MrpackIndexTooLarge { max_bytes } => Self::MrpackIndexTooLarge {
                max_bytes: max_bytes.to_string(),
            },
            R::MrpackMalformedIndex(err) => Self::MrpackMalformedIndex { cause: err.into() },
            R::MrpackTooManyFiles { count, max } => Self::MrpackTooManyFiles {
                count: count.to_string(),
                max: max.to_string(),
            },
            R::MrpackNotMinecraft { game } => Self::MrpackNotMinecraft { game: game.clone() },
            R::MrpackInvalidHash(_) => Self::MrpackInvalidHash,
            R::MrpackNoAllowedDownload { path } => {
                Self::MrpackNoAllowedDownload { path: path.clone() }
            }
            R::Provider(text) => Self::Provider {
                cause: CauseDto::unknown(text),
            },
        }
    }
}

/// Why content could not be downloaded.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentDownloadReasonDto {
    NoFiles { version_id: String },
    NoVersions { project_id: String },
    NotAModpackFile { file_name: String },
    Download { url: String, cause: CauseDto },
    Other { cause: CauseDto },
}

impl From<&ContentDownloadFailure> for ContentDownloadReasonDto {
    fn from(value: &ContentDownloadFailure) -> Self {
        match value {
            ContentDownloadFailure::NoFiles { version_id } => Self::NoFiles {
                version_id: version_id.clone(),
            },
            ContentDownloadFailure::NoVersions { project_id } => Self::NoVersions {
                project_id: project_id.clone(),
            },
            ContentDownloadFailure::NotAModpackFile { file_name } => Self::NotAModpackFile {
                file_name: file_name.clone(),
            },
            ContentDownloadFailure::Download { url, source } => Self::Download {
                url: without_query(url),
                cause: source_cause(source),
            },
            ContentDownloadFailure::Other(text) => Self::Other {
                cause: CauseDto::unknown(text),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::features::instance::InstanceError;
    use serde_json::{Value, json};

    use crate::core::infra::tauri::api::dtos::FrontendError;

    fn to_json(err: &InstanceError) -> Value {
        match FrontendError::from(err) {
            FrontendError::Instance(dto) => serde_json::to_value(dto).unwrap(),
            other => panic!("not an instance error: {other:?}"),
        }
    }

    fn provider(reason: ContentProviderRejection) -> Value {
        to_json(&InstanceError::ContentProviderError { reason })
    }

    fn download(failure: ContentDownloadFailure) -> Value {
        to_json(&InstanceError::ContentDownloadError(failure))
    }

    fn provider_id(name: &str) -> aether_core::features::instance::ProviderId {
        aether_core::features::instance::ProviderId::from_parts("aether", name).unwrap()
    }

    #[test]
    fn should_serialize_a_plan_for_the_wrong_target() {
        let value = provider(ContentProviderRejection::WrongTarget {
            provider_id: provider_id("modrinth"),
        });

        assert_eq!(
            value,
            json!({"code": "CONTENT_PROVIDER_ERROR", "payload": {"reason": {"code": "WRONG_TARGET", "payload": {"provider_id": "aether#modrinth"}}}})
        );
    }

    #[test]
    fn should_serialize_a_pack_of_another_provider() {
        let value = provider(ContentProviderRejection::ForeignPack {
            provider_id: provider_id("modrinth"),
            pack_provider_id: provider_id("other"),
        });

        assert_eq!(
            value["payload"]["reason"],
            json!({"code": "FOREIGN_PACK", "payload": {"provider_id": "aether#modrinth", "pack_provider_id": "aether#other"}})
        );
    }

    #[test]
    fn should_serialize_the_limits_of_a_plan_as_text() {
        assert_eq!(
            provider(ContentProviderRejection::TooManyFiles { count: 7, max: 5 })["payload"]["reason"],
            json!({"code": "TOO_MANY_FILES", "payload": {"count": "7", "max": "5"}})
        );
        assert_eq!(
            provider(ContentProviderRejection::TooManyOverrides { max: 9 })["payload"]["reason"],
            json!({"code": "TOO_MANY_OVERRIDES", "payload": {"max": "9"}})
        );
    }

    #[test]
    fn should_serialize_a_damaged_overrides_archive_with_an_io_cause() {
        let err = std::io::Error::from(std::io::ErrorKind::InvalidData);

        assert_eq!(
            provider(ContentProviderRejection::InvalidOverridesArchive(err))["payload"]["reason"],
            json!({"code": "INVALID_OVERRIDES_ARCHIVE", "payload": {"cause": {"type": "io", "kind": "invalidData"}}})
        );
    }

    #[test]
    fn should_serialize_a_version_of_another_project() {
        assert_eq!(
            provider(ContentProviderRejection::VersionOfAnotherProject {
                version_id: "v".into(),
                project_id: "p".into(),
                content_id: "c".into(),
            })["payload"]["reason"],
            json!({"code": "VERSION_OF_ANOTHER_PROJECT", "payload": {"version_id": "v", "project_id": "p", "content_id": "c"}})
        );
    }

    #[test]
    fn should_serialize_a_single_file_asked_as_a_modpack() {
        assert_eq!(
            provider(ContentProviderRejection::NotAModpack {
                title: "Sodium".into(),
                content_type: aether_core::features::instance::ContentType::Mod,
            })["payload"]["reason"],
            json!({"code": "NOT_A_MODPACK", "payload": {"title": "Sodium", "content_type": "mod"}})
        );
    }

    #[test]
    fn should_serialize_a_version_without_a_game_version() {
        assert_eq!(
            provider(ContentProviderRejection::NoGameVersion {
                version_id: "v".into()
            })["payload"]["reason"],
            json!({"code": "NO_GAME_VERSION", "payload": {"version_id": "v"}})
        );
    }

    #[test]
    fn should_serialize_an_unknown_project_type() {
        assert_eq!(
            provider(ContentProviderRejection::UnknownProjectType {
                project_type: "x".into()
            })["payload"]["reason"],
            json!({"code": "UNKNOWN_PROJECT_TYPE", "payload": {"project_type": "x"}})
        );
    }

    #[test]
    fn should_serialize_a_bad_search_with_the_text_of_the_mapper_as_unknown() {
        let source: BoxedSource = "bad page".into();

        assert_eq!(
            provider(ContentProviderRejection::InvalidSearchParams(source))["payload"]["reason"],
            json!({"code": "INVALID_SEARCH_PARAMS", "payload": {"cause": {"type": "unknown", "details": "bad page"}}})
        );
        assert_eq!(
            provider(ContentProviderRejection::InvalidSearchResponse("x".into()))["payload"]["reason"]
                ["code"],
            "INVALID_SEARCH_RESPONSE"
        );
        assert_eq!(
            provider(ContentProviderRejection::Mapping("x".into()))["payload"]["reason"]["code"],
            "MAPPING"
        );
    }

    #[test]
    fn should_serialize_a_mrpack_that_cannot_be_parsed_with_a_parse_cause() {
        let err = serde_json::from_str::<u8>("x").unwrap_err();

        let value = provider(ContentProviderRejection::MrpackMalformedIndex(err));

        assert_eq!(value["payload"]["reason"]["code"], "MRPACK_MALFORMED_INDEX");
        assert_eq!(
            value["payload"]["reason"]["payload"]["cause"]["type"],
            "parse"
        );
    }

    #[test]
    fn should_serialize_the_io_failures_of_a_mrpack() {
        let io = || std::io::Error::from(std::io::ErrorKind::NotFound);
        let cause = json!({"cause": {"type": "io", "kind": "notFound"}});

        for (reason, code) in [
            (
                ContentProviderRejection::MrpackOpenFailed(io()),
                "MRPACK_OPEN_FAILED",
            ),
            (
                ContentProviderRejection::MrpackInvalidArchive(io()),
                "MRPACK_INVALID_ARCHIVE",
            ),
            (
                ContentProviderRejection::MrpackReadFailed(io()),
                "MRPACK_READ_FAILED",
            ),
        ] {
            assert_eq!(
                provider(reason)["payload"]["reason"],
                json!({"code": code, "payload": cause})
            );
        }
    }

    #[test]
    fn should_serialize_the_refusals_of_a_mrpack_index() {
        assert_eq!(
            provider(ContentProviderRejection::MrpackMissingIndex)["payload"]["reason"],
            json!({"code": "MRPACK_MISSING_INDEX"})
        );
        assert_eq!(
            provider(ContentProviderRejection::MrpackIndexTooLarge { max_bytes: 10 })["payload"]["reason"],
            json!({"code": "MRPACK_INDEX_TOO_LARGE", "payload": {"max_bytes": "10"}})
        );
        assert_eq!(
            provider(ContentProviderRejection::MrpackTooManyFiles { count: 3, max: 2 })["payload"]
                ["reason"],
            json!({"code": "MRPACK_TOO_MANY_FILES", "payload": {"count": "3", "max": "2"}})
        );
        assert_eq!(
            provider(ContentProviderRejection::MrpackNotMinecraft { game: "x".into() })["payload"]
                ["reason"],
            json!({"code": "MRPACK_NOT_MINECRAFT", "payload": {"game": "x"}})
        );
        assert_eq!(
            provider(ContentProviderRejection::MrpackNoAllowedDownload {
                path: "mods/a.jar".into()
            })["payload"]["reason"],
            json!({"code": "MRPACK_NO_ALLOWED_DOWNLOAD", "payload": {"path": "mods/a.jar"}})
        );
    }

    #[test]
    fn should_serialize_a_plugin_provider_failure_with_its_text_as_unknown() {
        assert_eq!(
            provider(ContentProviderRejection::Provider("boom".into()))["payload"]["reason"],
            json!({"code": "PROVIDER", "payload": {"cause": {"type": "unknown", "details": "boom"}}})
        );
    }

    #[test]
    fn should_serialize_a_version_without_files() {
        assert_eq!(
            download(ContentDownloadFailure::NoFiles {
                version_id: "v".into()
            }),
            json!({"code": "CONTENT_DOWNLOAD_ERROR", "payload": {"reason": {"code": "NO_FILES", "payload": {"version_id": "v"}}}})
        );
        assert_eq!(
            download(ContentDownloadFailure::NoVersions {
                project_id: "p".into()
            })["payload"]["reason"],
            json!({"code": "NO_VERSIONS", "payload": {"project_id": "p"}})
        );
    }

    #[test]
    fn should_serialize_a_file_that_is_not_a_modpack() {
        assert_eq!(
            download(ContentDownloadFailure::NotAModpackFile {
                file_name: "a.jar".into()
            })["payload"]["reason"],
            json!({"code": "NOT_A_MODPACK_FILE", "payload": {"file_name": "a.jar"}})
        );
    }

    #[test]
    fn should_serialize_a_failed_download_with_the_url_without_its_query() {
        let source = RequestError::Status {
            status: 404,
            url: "https://cdn.example.com/a.jar?token=secret".into(),
        };

        let value = download(ContentDownloadFailure::download(
            "https://cdn.example.com/a.jar?token=secret",
            source,
        ));

        assert_eq!(
            value["payload"]["reason"],
            json!({"code": "DOWNLOAD", "payload": {"url": "https://cdn.example.com/a.jar", "cause": {"type": "httpStatus", "status": 404, "url": "https://cdn.example.com/a.jar"}}})
        );
    }

    #[test]
    fn should_serialize_a_download_refused_by_a_policy_as_unknown() {
        let value = download(ContentDownloadFailure::download(
            "https://x.test/a",
            "denied",
        ));

        assert_eq!(
            value["payload"]["reason"]["payload"]["cause"],
            json!({"type": "unknown", "details": "denied"})
        );
    }

    #[test]
    fn should_serialize_the_text_of_an_api_client_as_unknown() {
        assert_eq!(
            download(ContentDownloadFailure::Other("offline".into()))["payload"]["reason"],
            json!({"code": "OTHER", "payload": {"cause": {"type": "unknown", "details": "offline"}}})
        );
    }
}
