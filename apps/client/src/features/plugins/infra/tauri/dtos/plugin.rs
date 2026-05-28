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

impl From<aether_core::features::plugins::PluginDto> for PluginDto {
    fn from(value: aether_core::features::plugins::PluginDto) -> Self {
        Self {
            manifest: value.manifest.into(),
            capabilities: value.capabilities.map(Into::into),
            state: value.state.into(),
        }
    }
}

impl From<aether_core::features::plugins::PluginDtoState> for PluginDtoState {
    fn from(value: aether_core::features::plugins::PluginDtoState) -> Self {
        match value {
            aether_core::features::plugins::PluginDtoState::NotLoaded => Self::NotLoaded,
            aether_core::features::plugins::PluginDtoState::Loading => Self::Loading,
            aether_core::features::plugins::PluginDtoState::Loaded => Self::Loaded,
            aether_core::features::plugins::PluginDtoState::Unloading => Self::Unloading,
            aether_core::features::plugins::PluginDtoState::Failed => Self::Failed,
        }
    }
}
