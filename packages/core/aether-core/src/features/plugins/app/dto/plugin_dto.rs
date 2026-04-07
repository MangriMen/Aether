use crate::features::plugins::{Plugin, PluginCapabilities, PluginManifest, PluginState};
use dashmap::mapref::{multiple::RefMulti as DashMapRefMulti, one::Ref as DashMapRef};
use serde::{Deserialize, Serialize};

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
    pub manifest: PluginManifest,
    pub capabilities: Option<PluginCapabilities>,
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
            manifest: value.manifest.clone(),
            capabilities: value.capabilities.clone(),
            state: (&value.state).into(),
        }
    }
}

impl From<DashMapRef<'_, String, Plugin>> for PluginDto {
    fn from(value: DashMapRef<'_, String, Plugin>) -> Self {
        Self {
            manifest: value.manifest.clone(),
            capabilities: value.capabilities.clone(),
            state: (&value.state).into(),
        }
    }
}

impl From<DashMapRefMulti<'_, String, Plugin>> for PluginDto {
    fn from(value: DashMapRefMulti<'_, String, Plugin>) -> Self {
        Self {
            manifest: value.manifest.clone(),
            capabilities: value.capabilities.clone(),
            state: (&value.state).into(),
        }
    }
}
