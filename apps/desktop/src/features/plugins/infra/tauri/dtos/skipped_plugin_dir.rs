use aether_core::features::plugins::{SkippedPluginDir, SkippedPluginDirReason};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Translated by its `code`; the payload fills in the message.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkippedPluginDirReasonDto {
    InvalidName,
    LoadFailed { details: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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
                SkippedPluginDirReason::LoadFailed { details } => {
                    SkippedPluginDirReasonDto::LoadFailed { details }
                }
            },
        }
    }
}
