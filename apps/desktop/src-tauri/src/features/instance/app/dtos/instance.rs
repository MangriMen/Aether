use aether_core::features::instance::Instance;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::{
    instance::{InstanceInstallStageDto, PackInfoDto},
    minecraft::{LoaderVersionPreferenceDto, ModLoaderDto},
    settings::{HooksDto, MemorySettingsDto, WindowSizeDto},
};

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct InstanceDto {
    pub id: String,

    pub name: String,
    pub icon_path: Option<String>,

    pub install_stage: InstanceInstallStageDto,

    // Main minecraft metadata
    pub game_version: String,
    pub loader: ModLoaderDto,
    pub loader_version: Option<LoaderVersionPreferenceDto>,

    // Launch arguments
    pub java_path: Option<String>,
    pub launch_args: Option<Vec<String>>,
    pub env_vars: Option<Vec<(String, String)>>,

    // Minecraft runtime settings
    pub memory: Option<MemorySettingsDto>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSizeDto>,

    // Additional information
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    pub time_played: u64,
    pub recent_time_played: u64,

    pub hooks: HooksDto,

    pub pack_info: Option<PackInfoDto>,
}

impl From<Instance> for InstanceDto {
    fn from(value: Instance) -> Self {
        Self {
            id: value.id,
            name: value.name,
            icon_path: value.icon_path,
            install_stage: value.install_stage.into(),
            game_version: value.game_version,
            loader: value.loader.into(),
            loader_version: value.loader_version.map(std::convert::Into::into),
            java_path: value.java_path,
            launch_args: value.launch_args,
            env_vars: value.env_vars,
            memory: value.memory.map(std::convert::Into::into),
            force_fullscreen: value.force_fullscreen,
            game_resolution: value.game_resolution.map(std::convert::Into::into),
            created: value.created,
            modified: value.modified,
            last_played: value.last_played,
            time_played: value.time_played,
            recent_time_played: value.recent_time_played,
            hooks: value.hooks.into(),
            pack_info: value.pack_info.map(std::convert::Into::into),
        }
    }
}
