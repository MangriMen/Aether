use aether_core::features::instance::{ContentDownloadFailure, ContentProviderRejection};
use serde::Serialize;
use specta::Type;

use super::{CauseDto, PaginationFieldDto, without_query};
use crate::features::instance::infra::ContentTypeDto;

/// Why a content provider's answer, or the request made to it, was refused. The codes are the
/// same for every provider.
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
    VersionOfAnotherContent {
        version_id: String,
        owner_id: String,
        content_id: String,
    },
    NotAModpack {
        title: String,
        content_type: ContentTypeDto,
    },
    NoGameVersion {
        version_id: String,
    },
    UnknownContentType {
        content_type: String,
    },
    InvalidRequest {
        cause: CauseDto,
    },
    Request {
        cause: CauseDto,
    },
    ResponseInvalidId {
        value: String,
    },
    ResponseInvalidUrl {
        url: String,
    },
    ResponseInvalidPagination {
        field: PaginationFieldDto,
        value: String,
    },
    PackUnreadable {
        cause: CauseDto,
    },
    PackInvalidArchive {
        cause: CauseDto,
    },
    PackNoIndex {
        index: String,
    },
    PackIndexTooLarge {
        max_bytes: String,
    },
    PackIndexMalformed {
        cause: CauseDto,
    },
    PackWrongGame {
        game: String,
    },
    PackOutsideProviderReach {
        path: String,
    },
    InvalidHash,
    NoAllowedDownload {
        path: String,
    },
    InvalidAnswer {
        handler: String,
        cause: CauseDto,
    },
    Provider {
        cause: CauseDto,
    },
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
            R::VersionOfAnotherContent {
                version_id,
                owner_id,
                content_id,
            } => Self::VersionOfAnotherContent {
                version_id: version_id.clone(),
                owner_id: owner_id.clone(),
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
            R::UnknownContentType { content_type } => Self::UnknownContentType {
                content_type: content_type.clone(),
            },
            R::InvalidRequest(source) => Self::InvalidRequest {
                cause: CauseDto::of_source(source.as_ref()),
            },
            R::Request(source) => Self::Request {
                cause: CauseDto::of_source(source.as_ref()),
            },
            R::ResponseInvalidId { value, .. } => Self::ResponseInvalidId {
                value: value.clone(),
            },
            R::ResponseInvalidUrl(err) => Self::ResponseInvalidUrl {
                url: without_query(&err.value),
            },
            R::ResponseInvalidPagination(err) => Self::ResponseInvalidPagination {
                field: err.field.into(),
                value: err.value.to_string(),
            },
            R::PackUnreadable(err) => Self::PackUnreadable { cause: err.into() },
            R::PackInvalidArchive(err) => Self::PackInvalidArchive { cause: err.into() },
            R::PackNoIndex { index } => Self::PackNoIndex {
                index: (*index).to_owned(),
            },
            R::PackIndexTooLarge { max_bytes } => Self::PackIndexTooLarge {
                max_bytes: max_bytes.to_string(),
            },
            R::PackIndexMalformed(source) => Self::PackIndexMalformed {
                cause: CauseDto::of_source(source.as_ref()),
            },
            R::PackWrongGame { game } => Self::PackWrongGame { game: game.clone() },
            R::PackOutsideProviderReach { path } => Self::PackOutsideProviderReach {
                path: path.display().to_string(),
            },
            R::InvalidHash(_) => Self::InvalidHash,
            R::NoAllowedDownload { path } => Self::NoAllowedDownload { path: path.clone() },
            R::InvalidAnswer { handler, source } => Self::InvalidAnswer {
                handler: handler.clone(),
                cause: CauseDto::of_source(source.as_ref()),
            },
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
    NoVersions { content_id: String },
    NotAModpackFile { file_name: String },
    Download { url: String, cause: CauseDto },
    NoNetworkPolicy { plugin_id: String },
}

impl From<&ContentDownloadFailure> for ContentDownloadReasonDto {
    fn from(value: &ContentDownloadFailure) -> Self {
        match value {
            ContentDownloadFailure::NoFiles { version_id } => Self::NoFiles {
                version_id: version_id.clone(),
            },
            ContentDownloadFailure::NoVersions { content_id } => Self::NoVersions {
                content_id: content_id.clone(),
            },
            ContentDownloadFailure::NotAModpackFile { file_name } => Self::NotAModpackFile {
                file_name: file_name.clone(),
            },
            ContentDownloadFailure::Download { url, source } => Self::Download {
                url: without_query(url),
                cause: CauseDto::of_source(source.as_ref()),
            },
            ContentDownloadFailure::NoNetworkPolicy { plugin_id } => Self::NoNetworkPolicy {
                plugin_id: plugin_id.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::features::instance::{BoxedSource, InstanceError, InvalidContentId, PageSize};
    use aether_core::shared::error::shared_source;
    use aether_core::shared::request_client::RequestError;
    use aether_core::shared::web_url::HttpsUrl;
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
            provider(ContentProviderRejection::InvalidOverridesArchive(
                err.into()
            ))["payload"]["reason"],
            json!({"code": "INVALID_OVERRIDES_ARCHIVE", "payload": {"cause": {"type": "io", "kind": "invalidData"}}})
        );
    }

    #[test]
    fn should_serialize_a_version_of_another_content() {
        assert_eq!(
            provider(ContentProviderRejection::VersionOfAnotherContent {
                version_id: "v".into(),
                owner_id: "p".into(),
                content_id: "c".into(),
            })["payload"]["reason"],
            json!({"code": "VERSION_OF_ANOTHER_CONTENT", "payload": {"version_id": "v", "owner_id": "p", "content_id": "c"}})
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
    fn should_serialize_an_unknown_content_type() {
        assert_eq!(
            provider(ContentProviderRejection::UnknownContentType {
                content_type: "x".into()
            })["payload"]["reason"],
            json!({"code": "UNKNOWN_CONTENT_TYPE", "payload": {"content_type": "x"}})
        );
    }

    #[test]
    fn should_serialize_a_request_that_cannot_be_built_with_its_text_as_unknown() {
        let source: BoxedSource = "bad page".into();

        assert_eq!(
            provider(ContentProviderRejection::InvalidRequest(source.into()))["payload"]["reason"],
            json!({"code": "INVALID_REQUEST", "payload": {"cause": {"type": "unknown", "details": "bad page"}}})
        );
    }

    #[test]
    fn should_serialize_a_failed_request_with_the_category_of_the_client_error() {
        let source = RequestError::Status {
            status: 503,
            url: "https://api.example.com/v2/project/x?key=secret".into(),
        };

        assert_eq!(
            provider(ContentProviderRejection::Request(std::sync::Arc::new(
                source
            )))["payload"]["reason"],
            json!({"code": "REQUEST", "payload": {"cause": {"type": "httpStatus", "status": 503, "url": "https://api.example.com/v2/project/x"}}})
        );
    }

    #[test]
    fn should_serialize_the_invalid_values_of_a_response() {
        assert_eq!(
            provider(
                InvalidContentId {
                    kind: "content id",
                    value: "a/b".into(),
                }
                .into()
            )["payload"]["reason"],
            json!({"code": "RESPONSE_INVALID_ID", "payload": {"value": "a/b"}})
        );
        assert_eq!(
            provider(
                HttpsUrl::parse("http://cdn.example.com/a.jar?token=1")
                    .unwrap_err()
                    .into()
            )["payload"]["reason"],
            json!({"code": "RESPONSE_INVALID_URL", "payload": {"url": "http://cdn.example.com/a.jar"}})
        );
        assert_eq!(
            provider(PageSize::parse(0).unwrap_err().into())["payload"]["reason"],
            json!({"code": "RESPONSE_INVALID_PAGINATION", "payload": {"field": "PAGE_SIZE", "value": "0"}})
        );
    }

    #[test]
    fn should_serialize_a_pack_index_that_cannot_be_parsed_with_a_parse_cause() {
        let err = serde_json::from_str::<u8>("x").unwrap_err();

        let value = provider(ContentProviderRejection::PackIndexMalformed(
            std::sync::Arc::new(err),
        ));

        assert_eq!(value["payload"]["reason"]["code"], "PACK_INDEX_MALFORMED");
        assert_eq!(
            value["payload"]["reason"]["payload"]["cause"]["type"],
            "parse"
        );
    }

    #[test]
    fn should_serialize_the_io_failures_of_a_pack() {
        let io = || std::io::Error::from(std::io::ErrorKind::NotFound);
        let cause = json!({"cause": {"type": "io", "kind": "notFound"}});

        for (reason, code) in [
            (
                ContentProviderRejection::PackUnreadable(io().into()),
                "PACK_UNREADABLE",
            ),
            (
                ContentProviderRejection::PackInvalidArchive(io().into()),
                "PACK_INVALID_ARCHIVE",
            ),
        ] {
            assert_eq!(
                provider(reason)["payload"]["reason"],
                json!({"code": code, "payload": cause})
            );
        }
    }

    #[test]
    fn should_serialize_the_refusals_of_a_pack_index() {
        assert_eq!(
            provider(ContentProviderRejection::PackNoIndex {
                index: "modrinth.index.json"
            })["payload"]["reason"],
            json!({"code": "PACK_NO_INDEX", "payload": {"index": "modrinth.index.json"}})
        );
        assert_eq!(
            provider(ContentProviderRejection::PackIndexTooLarge { max_bytes: 10 })["payload"]["reason"],
            json!({"code": "PACK_INDEX_TOO_LARGE", "payload": {"max_bytes": "10"}})
        );
        assert_eq!(
            provider(ContentProviderRejection::PackWrongGame { game: "x".into() })["payload"]["reason"],
            json!({"code": "PACK_WRONG_GAME", "payload": {"game": "x"}})
        );
        assert_eq!(
            provider(ContentProviderRejection::NoAllowedDownload {
                path: "mods/a.jar".into()
            })["payload"]["reason"],
            json!({"code": "NO_ALLOWED_DOWNLOAD", "payload": {"path": "mods/a.jar"}})
        );
    }

    #[test]
    fn should_serialize_a_pack_a_plugin_cannot_read_with_its_path() {
        let path = std::path::PathBuf::from("pack.mrpack");

        assert_eq!(
            provider(ContentProviderRejection::PackOutsideProviderReach { path })["payload"]["reason"],
            json!({"code": "PACK_OUTSIDE_PROVIDER_REACH", "payload": {"path": "pack.mrpack"}})
        );
    }

    #[test]
    fn should_serialize_an_invalid_answer_of_a_plugin_with_the_handler_and_its_cause() {
        assert_eq!(
            provider(ContentProviderRejection::InvalidAnswer {
                handler: "cf_resolve_file".into(),
                source: shared_source("url is not https"),
            })["payload"]["reason"],
            json!({"code": "INVALID_ANSWER", "payload": {
                "handler": "cf_resolve_file",
                "cause": {"type": "unknown", "details": "url is not https"},
            }})
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
                content_id: "p".into()
            })["payload"]["reason"],
            json!({"code": "NO_VERSIONS", "payload": {"content_id": "p"}})
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
    fn should_serialize_a_download_of_a_plugin_without_a_network_policy() {
        assert_eq!(
            download(ContentDownloadFailure::NoNetworkPolicy {
                plugin_id: "p".into()
            })["payload"]["reason"],
            json!({"code": "NO_NETWORK_POLICY", "payload": {"plugin_id": "p"}})
        );
    }
}
