use aether_core::shared::cache::AssetError;
use std::sync::Arc;

use aether_core::shared::error::{AnyhowSource, DatabaseError};
use aether_core::shared::io::{IoError, IoErrorKind};
use aether_core::shared::request_client::RequestError;
use serde::Serialize;
use specta::Type;

use super::IoErrorKindDto;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NetworkErrorKindDto {
    Timeout,
    Connect,
    Redirect,
    Body,
}

/// Why a command failed, in a form the frontend can translate without seeing raw error text.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CauseDto {
    Io {
        kind: IoErrorKindDto,
    },
    Network {
        kind: NetworkErrorKindDto,
        url: Option<String>,
    },
    HttpStatus {
        status: u16,
        url: String,
    },
    Parse {
        line: Option<u32>,
        column: Option<u32>,
    },
    Database,
    /// The only place raw error text reaches the frontend.
    Unknown {
        details: String,
    },
}

impl CauseDto {
    pub fn unknown(details: impl std::fmt::Display) -> Self {
        Self::Unknown {
            details: details.to_string(),
        }
    }

    /// The category of an error the core passes on untyped: the first error in its chain of a
    /// kind with a category, else its text.
    pub fn of_source(source: &(dyn std::error::Error + 'static)) -> Self {
        let mut current = Some(source);
        while let Some(err) = current {
            if let Some(err) = err.downcast_ref::<RequestError>() {
                return err.into();
            }
            if let Some(err) = err.downcast_ref::<reqwest::Error>() {
                return err.into();
            }
            if let Some(err) = err.downcast_ref::<std::io::Error>() {
                return err.into();
            }
            // The library errors an error shares between its clones.
            if let Some(err) = err.downcast_ref::<Arc<std::io::Error>>() {
                return err.into();
            }
            if let Some(err) = err.downcast_ref::<Arc<reqwest::Error>>() {
                return err.into();
            }
            if let Some(err) = err.downcast_ref::<Arc<serde_json::Error>>() {
                return err.into();
            }
            if let Some(err) = err.downcast_ref::<AnyhowSource>() {
                return super::request_error::middleware_cause(err.inner());
            }
            if let Some(err) = err.downcast_ref::<IoError>() {
                return err.into();
            }
            if let Some(err) = err.downcast_ref::<serde_json::Error>() {
                return err.into();
            }
            if let Some(err) = err.downcast_ref::<toml::de::Error>() {
                return err.into();
            }
            if err.downcast_ref::<DatabaseError>().is_some() {
                return Self::Database;
            }
            current = err.source();
        }
        Self::unknown(source)
    }
}

impl From<&std::io::Error> for CauseDto {
    fn from(value: &std::io::Error) -> Self {
        match IoErrorKindDto::from_kind(IoErrorKind::of(value)) {
            Some(kind) => Self::Io { kind },
            None => Self::unknown(value),
        }
    }
}

// The library errors the core shares between the clones of an error.
impl From<&Arc<std::io::Error>> for CauseDto {
    fn from(value: &Arc<std::io::Error>) -> Self {
        value.as_ref().into()
    }
}

impl From<&Arc<reqwest::Error>> for CauseDto {
    fn from(value: &Arc<reqwest::Error>) -> Self {
        value.as_ref().into()
    }
}

impl From<&Arc<serde_json::Error>> for CauseDto {
    fn from(value: &Arc<serde_json::Error>) -> Self {
        value.as_ref().into()
    }
}

impl From<&AssetError> for CauseDto {
    /// An icon refused for its size or format has no category of its own: its text is kept.
    fn from(value: &AssetError) -> Self {
        match value {
            AssetError::ReadSource(err) | AssetError::Write(err) | AssetError::Remove(err) => {
                err.into()
            }
            AssetError::TooLarge { .. } | AssetError::NotAnImage => Self::unknown(value),
        }
    }
}

impl From<&DatabaseError> for CauseDto {
    fn from(_: &DatabaseError) -> Self {
        Self::Database
    }
}

impl From<&IoError> for CauseDto {
    fn from(value: &IoError) -> Self {
        match value {
            IoError::IoPathError { source, .. } | IoError::IoError(source) => source.into(),
            IoError::SerializationError(_) | IoError::DeserializationError(_) => Self::Parse {
                line: None,
                column: None,
            },
        }
    }
}

/// A URL as text without its query, fragment and credentials, which can carry secrets.
pub fn without_query(url: &str) -> String {
    if let Ok(parsed) = reqwest::Url::parse(url) {
        return sanitize_url(&parsed);
    }

    // Not parseable, so cut by hand: everything before `?`/`#`, minus `user:pass@`.
    let base = url.split(['?', '#']).next().unwrap_or_default();
    let Some((scheme, rest)) = base.split_once("://") else {
        return base.to_owned();
    };
    let authority_end = rest.find('/').unwrap_or(rest.len());
    let without_credentials = match rest[..authority_end].rfind('@') {
        Some(at) => &rest[at + 1..],
        None => rest,
    };
    format!("{scheme}://{without_credentials}")
}

/// Drops the parts of a URL that can carry secrets or noise (credentials, query, fragment).
fn sanitize_url(url: &reqwest::Url) -> String {
    let mut url = url.clone();
    url.set_query(None);
    url.set_fragment(None);
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.to_string()
}

impl From<&reqwest::Error> for CauseDto {
    fn from(value: &reqwest::Error) -> Self {
        let url = value.url().map(sanitize_url);
        let network = |kind| Self::Network {
            kind,
            url: url.clone(),
        };

        if value.is_status() {
            return Self::HttpStatus {
                status: value.status().map_or(0, |status| status.as_u16()),
                url: url.unwrap_or_default(),
            };
        }
        if value.is_timeout() {
            return network(NetworkErrorKindDto::Timeout);
        }
        if value.is_connect() {
            return network(NetworkErrorKindDto::Connect);
        }
        if value.is_redirect() {
            return network(NetworkErrorKindDto::Redirect);
        }
        if value.is_body() || value.is_decode() {
            return network(NetworkErrorKindDto::Body);
        }

        let mut details = value.to_string();
        if let (Some(raw), Some(clean)) = (value.url(), url) {
            details = details.replace(raw.as_str(), &clean);
        }
        Self::Unknown { details }
    }
}

impl From<&serde_json::Error> for CauseDto {
    fn from(value: &serde_json::Error) -> Self {
        // serde_json reports line 0 when the error has no position (e.g. a custom message).
        let position = |n: usize| u32::try_from(n).ok().filter(|n| *n > 0);
        Self::Parse {
            line: position(value.line()),
            column: position(value.column()),
        }
    }
}

impl From<&toml::de::Error> for CauseDto {
    fn from(_: &toml::de::Error) -> Self {
        Self::Parse {
            line: None,
            column: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn to_json(cause: &CauseDto) -> serde_json::Value {
        serde_json::to_value(cause).unwrap()
    }

    #[test]
    fn should_serialize_io_cause_with_camel_case_tag() {
        let cause = CauseDto::Io {
            kind: IoErrorKindDto::StorageFull,
        };

        assert_eq!(
            to_json(&cause),
            json!({"type": "io", "kind": "storageFull"})
        );
    }

    #[test]
    fn should_serialize_http_status_cause() {
        let cause = CauseDto::HttpStatus {
            status: 404,
            url: "https://example.com/a".to_owned(),
        };

        assert_eq!(
            to_json(&cause),
            json!({"type": "httpStatus", "status": 404, "url": "https://example.com/a"})
        );
    }

    #[test]
    fn should_serialize_parse_cause() {
        let cause = CauseDto::Parse {
            line: Some(3),
            column: None,
        };

        assert_eq!(
            to_json(&cause),
            json!({"type": "parse", "line": 3, "column": null})
        );
    }

    #[test]
    fn should_serialize_unknown_cause() {
        assert_eq!(
            to_json(&CauseDto::unknown("boom")),
            json!({"type": "unknown", "details": "boom"})
        );
    }

    #[test]
    fn should_map_serde_json_error_to_parse_with_position() {
        let err = serde_json::from_str::<serde_json::Value>("{\n  \"a\": }").unwrap_err();

        let CauseDto::Parse { line, column } = CauseDto::from(&err) else {
            panic!("expected a parse cause");
        };

        assert_eq!(line, Some(2));
        assert!(column.is_some());
    }

    #[test]
    fn should_map_other_io_error_to_unknown() {
        let err = std::io::Error::other("weird");

        assert_eq!(
            to_json(&(&err).into()),
            json!({"type": "unknown", "details": "weird"})
        );
    }

    #[test]
    fn should_categorize_a_typed_error_found_in_the_chain_of_a_source() {
        #[derive(Debug, thiserror::Error)]
        #[error("retry budget spent")]
        struct Retry(#[source] std::io::Error);

        let source: aether_core::shared::error::BoxedSource =
            Box::new(Retry(std::io::Error::from(std::io::ErrorKind::TimedOut)));

        assert_eq!(
            to_json(&CauseDto::of_source(source.as_ref())),
            json!({"type": "io", "kind": "timedOut"})
        );
    }

    #[test]
    fn should_categorize_the_error_inside_a_shared_anyhow_source() {
        let source: aether_core::shared::error::SharedSource =
            std::sync::Arc::new(aether_core::shared::error::AnyhowSource::new(
                anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::TimedOut)),
            ));

        assert_eq!(
            to_json(&CauseDto::of_source(source.as_ref())),
            json!({"type": "io", "kind": "timedOut"})
        );
    }

    #[test]
    fn should_keep_the_text_of_a_source_without_a_category_as_unknown() {
        let source: aether_core::shared::error::BoxedSource = "denied".into();

        assert_eq!(
            to_json(&CauseDto::of_source(source.as_ref())),
            json!({"type": "unknown", "details": "denied"})
        );
    }

    #[test]
    fn should_map_serialization_error_to_parse() {
        let err = IoError::SerializationError("x".to_owned());

        assert_eq!(
            to_json(&(&err).into()),
            json!({"type": "parse", "line": null, "column": null})
        );
    }
}
