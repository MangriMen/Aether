use aether_core::shared::request_client::RequestError;
use serde::Serialize;
use specta::Type;

use super::CauseDto;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestErrorDto {
    Acquire,
    RequestSend { cause: CauseDto },
    Middleware { cause: CauseDto },
    Hash { actual: String, expected: String },
    Json { cause: CauseDto },
    Toml { cause: CauseDto },
    Parse,
    Status { status: u16, url: String },
    // `String`, not `u64`: specta refuses to export `u64` to TypeScript.
    TooLarge { limit: String },
    Io { path: String, cause: CauseDto },
}

/// The middleware error is an opaque `anyhow::Error`, so the typed causes it can wrap are looked up in its chain.
pub(super) fn middleware_cause(err: &anyhow::Error) -> CauseDto {
    for source in err.chain() {
        if let Some(err) = source.downcast_ref::<reqwest::Error>() {
            return err.into();
        }
        if let Some(err) = source.downcast_ref::<std::io::Error>() {
            return err.into();
        }
    }
    CauseDto::unknown(err)
}

impl From<&RequestError> for CauseDto {
    fn from(value: &RequestError) -> Self {
        match value {
            RequestError::RequestSend(err) => err.into(),
            RequestError::Middleware(err) => middleware_cause(err),
            RequestError::Json(err) => err.into(),
            RequestError::Toml(err) => err.into(),
            RequestError::Status { status, url } => Self::HttpStatus {
                status: *status,
                url: super::without_query(url),
            },
            RequestError::Io { source, .. } => source.into(),
            RequestError::Hash { .. }
            | RequestError::Parse(_)
            | RequestError::Acquire(_)
            | RequestError::TooLarge { .. } => Self::unknown(value),
        }
    }
}

impl From<&RequestError> for RequestErrorDto {
    fn from(value: &RequestError) -> Self {
        match value {
            RequestError::Acquire(_) => Self::Acquire,
            RequestError::RequestSend(err) => Self::RequestSend { cause: err.into() },
            RequestError::Middleware(err) => Self::Middleware {
                cause: middleware_cause(err),
            },
            RequestError::Hash { actual, expected } => Self::Hash {
                actual: actual.clone(),
                expected: expected.clone(),
            },
            RequestError::Json(err) => Self::Json { cause: err.into() },
            RequestError::Toml(err) => Self::Toml { cause: err.into() },
            RequestError::Parse(_) => Self::Parse,
            RequestError::Status { status, url } => Self::Status {
                status: *status,
                url: super::without_query(url),
            },
            RequestError::TooLarge { limit } => Self::TooLarge {
                limit: limit.to_string(),
            },
            RequestError::Io { path, source } => Self::Io {
                path: path.display().to_string(),
                cause: source.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    fn to_json(err: &RequestError) -> serde_json::Value {
        serde_json::to_value(RequestErrorDto::from(err)).unwrap()
    }

    #[test]
    fn should_strip_the_token_query_and_credentials_from_a_status_url() {
        let err = RequestError::Status {
            status: 403,
            url: "https://user:pass@example.com/a/b?token=secret#frag".to_owned(),
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "STATUS", "payload": {"status": 403, "url": "https://example.com/a/b"}})
        );
    }

    #[test]
    fn should_strip_the_query_and_credentials_from_an_unparseable_url() {
        assert_eq!(
            super::super::without_query("https://user:pass@exa mple.com/a?token=secret"),
            "https://exa mple.com/a"
        );
    }

    #[test]
    fn should_serialize_request_send_with_a_cause() {
        let dto = RequestErrorDto::RequestSend {
            cause: CauseDto::unknown("boom"),
        };

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "REQUEST_SEND", "payload": {"cause": {"type": "unknown", "details": "boom"}}})
        );
    }

    #[test]
    fn should_map_io_error_to_path_and_io_cause() {
        let err = RequestError::Io {
            path: PathBuf::from("a.jar"),
            source: std::io::Error::from(std::io::ErrorKind::NotFound),
        };

        assert_eq!(
            to_json(&err),
            json!({"code": "IO", "payload": {"path": "a.jar", "cause": {"type": "io", "kind": "notFound"}}})
        );
    }

    #[test]
    fn should_map_serde_json_error_to_parse_cause() {
        let err = RequestError::Json(serde_json::from_str::<u8>("x").unwrap_err());

        let value = to_json(&err);

        assert_eq!(value["code"], "JSON");
        assert_eq!(value["payload"]["cause"]["type"], "parse");
    }

    #[test]
    fn should_map_toml_error_to_parse_cause() {
        let err = RequestError::Toml(toml::from_str::<toml::Table>("a = ").unwrap_err());

        assert_eq!(
            to_json(&err),
            json!({"code": "TOML", "payload": {"cause": {"type": "parse", "line": null, "column": null}}})
        );
    }

    #[test]
    fn should_map_a_reqwest_error_built_without_network() {
        let reqwest_err = reqwest::Client::new().get("http://").build().unwrap_err();
        let err = RequestError::RequestSend(reqwest_err);

        let value = to_json(&err);

        assert_eq!(value["code"], "REQUEST_SEND");
        assert!(value["payload"]["cause"]["type"].is_string());
    }

    #[test]
    fn should_find_a_typed_cause_in_the_middleware_chain() {
        let io = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        let err = RequestError::Middleware(anyhow::Error::new(io).context("wrapped"));

        assert_eq!(
            to_json(&err),
            json!({"code": "MIDDLEWARE", "payload": {"cause": {"type": "io", "kind": "permissionDenied"}}})
        );
    }
}
