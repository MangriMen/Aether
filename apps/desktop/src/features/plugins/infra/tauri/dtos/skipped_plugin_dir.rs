use aether_core::features::plugins::{SkippedPluginDir, SkippedPluginDirReason};
use serde::Serialize;
use specta::Type;

use crate::core::FrontendError;

/// Translated by its `code`; the payload fills in the message.
#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkippedPluginDirReasonDto {
    InvalidName,
    LoadFailed { error: FrontendError },
}

#[derive(Debug, Serialize, Type)]
pub struct SkippedPluginDirDto {
    pub dir_name: String,
    pub reason: SkippedPluginDirReasonDto,
}

impl From<SkippedPluginDir> for SkippedPluginDirDto {
    fn from(value: SkippedPluginDir) -> Self {
        Self {
            dir_name: value.dir_name,
            reason: match value.reason {
                SkippedPluginDirReason::InvalidName => SkippedPluginDirReasonDto::InvalidName,
                SkippedPluginDirReason::LoadFailed { error } => {
                    SkippedPluginDirReasonDto::LoadFailed {
                        error: error.as_ref().into(),
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use aether_core::features::plugins::{ManifestError, PluginError};
    use serde_json::json;

    use super::*;

    fn skipped(reason: SkippedPluginDirReason) -> serde_json::Value {
        serde_json::to_value(SkippedPluginDirDto::from(SkippedPluginDir {
            dir_name: "broken".to_owned(),
            reason,
        }))
        .unwrap()
    }

    #[test]
    fn should_serialize_an_invalid_name_as_a_bare_code() {
        assert_eq!(
            skipped(SkippedPluginDirReason::InvalidName),
            json!({"dir_name": "broken", "reason": {"code": "INVALID_NAME"}})
        );
    }

    #[test]
    fn should_carry_the_typed_error_when_the_plugin_could_not_be_read() {
        let error = PluginError::Manifest(ManifestError::ReservedPluginId);

        assert_eq!(
            skipped(SkippedPluginDirReason::LoadFailed {
                error: Arc::new(error)
            }),
            json!({
                "dir_name": "broken",
                "reason": {
                    "code": "LOAD_FAILED",
                    "payload": {
                        "error": {
                            "type": "plugin",
                            "payload": {
                                "code": "MANIFEST",
                                "payload": { "error": { "code": "RESERVED_PLUGIN_ID" } },
                            },
                        },
                    },
                },
            })
        );
    }
}
