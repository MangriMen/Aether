use serde::{Deserialize, Serialize};
use specta::Type;

/// Provider source type for plugin installation.
/// Mirrors `PluginSourceType` from the domain layer for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PluginSourceTypeDto {
    GitHub,
    Local,
}

impl From<aether_core::features::plugins::PluginSourceType> for PluginSourceTypeDto {
    fn from(value: aether_core::features::plugins::PluginSourceType) -> Self {
        match value {
            aether_core::features::plugins::PluginSourceType::GitHub => Self::GitHub,
            aether_core::features::plugins::PluginSourceType::Local => Self::Local,
        }
    }
}

impl From<PluginSourceTypeDto> for aether_core::features::plugins::PluginSourceType {
    fn from(value: PluginSourceTypeDto) -> Self {
        match value {
            PluginSourceTypeDto::GitHub => Self::GitHub,
            PluginSourceTypeDto::Local => Self::Local,
        }
    }
}
