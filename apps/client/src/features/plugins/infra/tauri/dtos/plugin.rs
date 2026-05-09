use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::plugins::{PluginCapabilitiesDto, PluginManifestDto};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum PluginDtoState {
    NotLoaded,
    Loading,
    Loaded,
    Unloading,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PluginDto {
    pub manifest: PluginManifestDto,
    pub capabilities: Option<PluginCapabilitiesDto>,
    pub state: PluginDtoState,
}

impl From<aether_core::features::plugins::app::PluginDto> for PluginDto {
    fn from(value: aether_core::features::plugins::app::PluginDto) -> Self {
        Self {
            manifest: value.manifest.into(),
            capabilities: value.capabilities.map(Into::into),
            state: value.state.into(),
        }
    }
}

impl From<aether_core::features::plugins::app::PluginDtoState> for PluginDtoState {
    fn from(value: aether_core::features::plugins::app::PluginDtoState) -> Self {
        use aether_core::features::plugins::app::PluginDtoState;
        match value {
            PluginDtoState::NotLoaded => Self::NotLoaded,
            PluginDtoState::Loading => Self::Loading,
            PluginDtoState::Loaded => Self::Loaded,
            PluginDtoState::Unloading => Self::Unloading,
            PluginDtoState::Failed => Self::Failed,
        }
    }
}
