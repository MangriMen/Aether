use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::features::{
    instance::{Instance, InstanceInstallStage},
    minecraft::{LoaderVersionPreference, ModLoader},
    settings::{Hooks, MemorySettings, WindowSize},
};

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceV1 {
    pub id: String,

    pub name: String,
    pub icon_path: Option<String>,

    pub install_stage: InstanceInstallStage,

    // Main minecraft metadata
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,

    // Launch arguments
    pub java_path: Option<String>,
    pub launch_args: Option<Vec<String>>,
    pub env_vars: Option<Vec<(String, String)>>,

    // Minecraft runtime settings
    pub memory: Option<MemorySettings>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSize>,

    // Additional information
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    pub time_played: u64,
    pub recent_time_played: u64,

    pub hooks: Hooks,

    pub pack_info: Option<super::pack_info::PackInfoV1>,
}

impl From<InstanceV1> for Instance {
    fn from(v1: InstanceV1) -> Self {
        Self {
            id: v1.id,
            name: v1.name,
            icon_path: v1.icon_path,
            install_stage: v1.install_stage,
            game_version: v1.game_version,
            loader: v1.loader,
            loader_version: v1.loader_version,
            java_path: v1.java_path,
            launch_args: v1.launch_args,
            env_vars: v1.env_vars,
            memory: v1.memory,
            force_fullscreen: v1.force_fullscreen,
            game_resolution: v1.game_resolution,
            created: v1.created,
            modified: v1.modified,
            last_played: v1.last_played,
            time_played: v1.time_played,
            recent_time_played: v1.recent_time_played,
            hooks: v1.hooks,
            pack_info: v1.pack_info.map(Into::into),
        }
    }
}
