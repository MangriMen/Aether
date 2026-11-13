use aether_core::shared::io::{IoError, IoErrorKind};
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
}

impl From<&std::io::Error> for CauseDto {
    fn from(value: &std::io::Error) -> Self {
        match IoErrorKindDto::from_kind(IoErrorKind::of(value)) {
            Some(kind) => Self::Io { kind },
            None => Self::unknown(value),
        }
    }
}

impl From<&sqlx::Error> for CauseDto {
    fn from(_: &sqlx::Error) -> Self {
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

/// A URL as text without its query and fragment, which can carry secrets.
pub fn without_query(url: &str) -> String {
    url.split(['?', '#']).next().unwrap_or_default().to_owned()
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
    fn should_map_serialization_error_to_parse() {
        let err = IoError::SerializationError("x".to_owned());

        assert_eq!(
            to_json(&(&err).into()),
            json!({"type": "parse", "line": null, "column": null})
        );
    }
}
