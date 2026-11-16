use register_schema::RegisterSchema;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};

use super::HostError;

/// Starts the failure envelope inside the plugin's error string.
pub const PLUGIN_FAILURE_MARKER: &str = "aether-plugin-failure:";

/// A failure a plugin reports to the host with more than text.
///
/// To report it, a plugin sets the error of its call (`error_set`) to the marker
/// `aether-plugin-failure:` directly followed by this value as one line of JSON, and returns a
/// non-zero code. The host looks for the marker anywhere in the string, so a prefix added by a
/// plugin framework does no harm, and ignores whatever follows the JSON. A string with no valid
/// envelope is an ordinary failure with that text as its cause.
// Published as `schemas/plugin-failure.schema.json`. A plain `//` comment, so that it stays out
// of the schema `description` that plugin authors read; see `PluginManifestDto`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "PluginFailure")]
#[schema_category("plugin_api")]
#[schema_name("PluginFailure")]
pub struct PluginFailureDto {
    /// What the plugin says went wrong; shown as the cause when `host` is absent or not understood.
    pub message: String,
    /// The error a host function returned to the plugin, passed back untouched. Absent when the
    /// failure is the plugin's own. A value the host does not understand is ignored, the
    /// `message` still counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(schema_with = "host_error_schema")]
    pub host: Option<HostError>,
}

// The full `HostError` contract is not described by a schema yet, so only its envelope is:
// `type` names the variant, `payload` carries it. (The field's doc comment is the description.)
fn host_error_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "object",
        "properties": {
            "type": { "type": "string" },
            "payload": {}
        },
        "required": ["type"]
    })
}

impl PluginFailureDto {
    /// The string to set as the call's error.
    #[must_use]
    pub fn encode(&self) -> String {
        let json = serde_json::to_string(self).expect("a failure envelope always serializes");
        format!("{PLUGIN_FAILURE_MARKER}{json}")
    }

    /// The envelope in an error string, if it holds one. A `host` that is not a `HostError` this
    /// version knows is dropped, the `message` stays.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        #[derive(Deserialize)]
        struct Raw {
            message: String,
            #[serde(default)]
            host: Option<serde_json::Value>,
        }

        // Each marker in turn: one in a prefix may be followed by text, not by the envelope.
        raw.match_indices(PLUGIN_FAILURE_MARKER)
            .find_map(|(at, _)| {
                // A stream, so that whatever a framework appends after the JSON is not read.
                let json = &raw[at + PLUGIN_FAILURE_MARKER.len()..];
                let envelope = serde_json::Deserializer::from_str(json)
                    .into_iter::<Raw>()
                    .next()?
                    .ok()?;

                Some(Self {
                    message: envelope.message,
                    host: envelope
                        .host
                        .and_then(|host| serde_json::from_value(host).ok()),
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v0::PluginErrorDto;

    fn refused() -> PluginFailureDto {
        PluginFailureDto {
            message: "AccessViolation".to_owned(),
            host: Some(HostError::PluginError(PluginErrorDto::AccessViolation {
                plugin_id: "packwiz".to_owned(),
                path: "denied".to_owned(),
            })),
        }
    }

    fn denied_path(failure: &PluginFailureDto) -> Option<&str> {
        match failure.host.as_ref()? {
            HostError::PluginError(PluginErrorDto::AccessViolation { path, .. }) => Some(path),
            _ => None,
        }
    }

    #[test]
    fn round_trips() {
        let parsed = PluginFailureDto::parse(&refused().encode()).expect("envelope");

        assert_eq!(parsed.message, "AccessViolation");
        assert_eq!(denied_path(&parsed), Some("denied"));
    }

    #[test]
    fn found_after_a_prefix_and_before_a_tail() {
        let raw = format!("Error: {}\n\nCaused by: x", refused().encode());
        let parsed = PluginFailureDto::parse(&raw).expect("envelope");

        assert_eq!(denied_path(&parsed), Some("denied"));
    }

    #[test]
    fn a_message_without_the_host_error_parses() {
        let raw = format!(r#"{PLUGIN_FAILURE_MARKER}{{"message":"boom"}}"#);
        let parsed = PluginFailureDto::parse(&raw).expect("envelope");

        assert_eq!(parsed.message, "boom");
        assert!(parsed.host.is_none());
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let raw = format!(r#"{PLUGIN_FAILURE_MARKER}{{"message":"boom","later":1}}"#);

        assert!(PluginFailureDto::parse(&raw).is_some());
    }

    #[test]
    fn a_host_error_it_cannot_read_leaves_the_message() {
        for host in [r#"{"type":"bogus"}"#, r#"{"type":"coreError"}"#] {
            let raw = format!(r#"{PLUGIN_FAILURE_MARKER}{{"message":"hi","host":{host}}}"#);
            let parsed = PluginFailureDto::parse(&raw).expect("envelope");

            assert_eq!(parsed.message, "hi");
            assert!(parsed.host.is_none());
        }
    }

    #[test]
    fn a_later_marker_is_tried_when_the_first_is_no_envelope() {
        let raw = format!("{PLUGIN_FAILURE_MARKER} oops: {}", refused().encode());

        assert!(PluginFailureDto::parse(&raw).is_some_and(|f| f.host.is_some()));
    }

    #[test]
    fn text_that_is_no_envelope_is_none() {
        assert!(PluginFailureDto::parse("plain failure").is_none());
        assert!(PluginFailureDto::parse(&format!("{PLUGIN_FAILURE_MARKER}not json")).is_none());
        assert!(
            PluginFailureDto::parse(&format!("{PLUGIN_FAILURE_MARKER}{{\"host\":null}}")).is_none()
        );
    }
}
