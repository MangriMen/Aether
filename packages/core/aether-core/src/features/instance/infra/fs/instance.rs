use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    features::{
        instance::{Instance, InstanceInstallStage, InstanceSnapshot},
        minecraft::{LoaderVersionPreference, ModLoader},
        settings::{Hooks, MemorySettings, WindowSettings, WindowSize},
    },
    shared::Overridable,
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

    pub hooks: HooksV1,

    pub pack_info: Option<super::pack_info::PackInfoV1>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct HooksV1 {
    pub pre_launch: Option<String>,
    pub wrapper: Option<String>,
    pub post_exit: Option<String>,
}

impl From<InstanceV1> for Instance {
    fn from(v1: InstanceV1) -> Self {
        let snapshot = InstanceSnapshot {
            id: v1.id,

            name: v1.name,

            icon_path: v1.icon_path,

            install_stage: v1.install_stage,

            game_version: v1.game_version,
            loader: v1.loader,
            loader_version: v1.loader_version,

            java_path: Overridable::new(
                v1.java_path.clone().unwrap_or_default(),
                v1.java_path.is_some(),
            ),

            launch_args: Overridable::new(
                v1.launch_args.clone().unwrap_or_default(),
                v1.launch_args.is_some(),
            ),

            env_vars: Overridable::new(
                v1.env_vars.clone().unwrap_or_default(),
                v1.env_vars.is_some(),
            ),

            memory: Overridable::new(v1.memory.unwrap_or_default(), v1.memory.is_some()),

            window: Overridable::new(
                WindowSettings::new(
                    v1.force_fullscreen.unwrap_or_default(),
                    v1.game_resolution.unwrap_or_default(),
                ),
                v1.force_fullscreen.is_some() || v1.game_resolution.is_some(),
            ),

            created: v1.created,
            modified: v1.modified,
            last_played: v1.last_played,

            time_played: v1.time_played,
            recent_time_played: v1.recent_time_played,

            hooks: Overridable::new(
                v1.hooks.clone().into(),
                v1.hooks.pre_launch.is_some()
                    || v1.hooks.wrapper.is_some()
                    || v1.hooks.post_exit.is_some(),
            ),

            pack_info: v1.pack_info.map(Into::into),
        };

        Instance::from_snapshot(snapshot)
    }
}

impl From<HooksV1> for Hooks {
    fn from(value: HooksV1) -> Self {
        Self::new(
            value.pre_launch.unwrap_or_default(),
            value.wrapper.unwrap_or_default(),
            value.post_exit.unwrap_or_default(),
        )
    }
}
