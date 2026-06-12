use aether_core_plugin_api::v0::{PluginCapabilitiesDto, PluginManifestDto};
use dashmap::mapref::{multiple::RefMulti as DashMapRefMulti, one::Ref as DashMapRef};
use serde::{Deserialize, Serialize};

use crate::features::plugins::domain::{Plugin, PluginState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginDtoState {
    NotLoaded,
    Loading,
    Loaded,
    Unloading,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDto {
    pub manifest: PluginManifestDto,
    pub capabilities: Option<PluginCapabilitiesDto>,
    pub state: PluginDtoState,
}

impl From<&PluginState> for PluginDtoState {
    fn from(value: &PluginState) -> Self {
        match value {
            PluginState::NotLoaded => PluginDtoState::NotLoaded,
            PluginState::Loading => PluginDtoState::Loading,
            PluginState::Loaded(_) => PluginDtoState::Loaded,
            PluginState::Unloading => PluginDtoState::Unloading,
            PluginState::Failed(_) => PluginDtoState::Failed,
        }
    }
}

impl From<Plugin> for PluginDto {
    fn from(value: Plugin) -> Self {
        Self {
            manifest: value.manifest.into(),
            capabilities: value.capabilities.map(Into::into),
            state: (&value.state).into(),
        }
    }
}

impl From<DashMapRef<'_, String, Plugin>> for PluginDto {
    fn from(value: DashMapRef<'_, String, Plugin>) -> Self {
        Self {
            manifest: value.manifest.clone().into(),
            capabilities: value.capabilities.clone().map(Into::into),
            state: (&value.state).into(),
        }
    }
}

impl From<DashMapRefMulti<'_, String, Plugin>> for PluginDto {
    fn from(value: DashMapRefMulti<'_, String, Plugin>) -> Self {
        Self {
            manifest: value.manifest.clone().into(),
            capabilities: value.capabilities.clone().map(Into::into),
            state: (&value.state).into(),
        }
    }
}
