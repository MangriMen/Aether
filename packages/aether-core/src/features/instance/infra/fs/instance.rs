use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    features::{
        instance::{
            Instance, InstanceInstallStage, InstanceSnapshot,
            infra::fs::pack_info::{PackInfoV1, PackInfoV2},
        },
        minecraft::{LoaderVersionPreference, ModLoader},
        settings::{
            Hooks, WindowSettings,
            infra::{HooksV2, MemorySettingsV1, WindowSettingsV1, WindowSizeV1},
        },
    },
    shared::overridable::domain::Overridable,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoaderVersionPreferenceV1 {
    Latest,
    #[default]
    Stable,
    #[serde(untagged)]
    Exact(String),
}

impl From<LoaderVersionPreferenceV1> for LoaderVersionPreference {
    fn from(value: LoaderVersionPreferenceV1) -> Self {
        match value {
            LoaderVersionPreferenceV1::Latest => Self::Latest,
            LoaderVersionPreferenceV1::Stable => Self::Stable,
            LoaderVersionPreferenceV1::Exact(x) => Self::Exact(x),
        }
    }
}

impl From<LoaderVersionPreference> for LoaderVersionPreferenceV1 {
    fn from(value: LoaderVersionPreference) -> Self {
        match value {
            LoaderVersionPreference::Latest => Self::Latest,
            LoaderVersionPreference::Stable => Self::Stable,
            LoaderVersionPreference::Exact(x) => Self::Exact(x),
        }
    }
}

/// DTO for `ModLoader` serialization in FS storage.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderV1 {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl From<ModLoaderV1> for ModLoader {
    fn from(value: ModLoaderV1) -> Self {
        match value {
            ModLoaderV1::Vanilla => Self::Vanilla,
            ModLoaderV1::Forge => Self::Forge,
            ModLoaderV1::Fabric => Self::Fabric,
            ModLoaderV1::Quilt => Self::Quilt,
            ModLoaderV1::NeoForge => Self::NeoForge,
        }
    }
}

impl From<ModLoader> for ModLoaderV1 {
    fn from(value: ModLoader) -> Self {
        match value {
            ModLoader::Vanilla => Self::Vanilla,
            ModLoader::Forge => Self::Forge,
            ModLoader::Fabric => Self::Fabric,
            ModLoader::Quilt => Self::Quilt,
            ModLoader::NeoForge => Self::NeoForge,
        }
    }
}

/// DTO for `InstanceInstallStage` serialization in FS storage.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceInstallStageV1 {
    Installed,
    Installing,
    PackInstalling,
    NotInstalled,
}

impl From<InstanceInstallStageV1> for InstanceInstallStage {
    fn from(value: InstanceInstallStageV1) -> Self {
        match value {
            InstanceInstallStageV1::Installed => Self::Installed,
            InstanceInstallStageV1::Installing => Self::Installing,
            InstanceInstallStageV1::PackInstalling => Self::PackInstalling,
            InstanceInstallStageV1::NotInstalled => Self::NotInstalled,
        }
    }
}

impl From<InstanceInstallStage> for InstanceInstallStageV1 {
    fn from(value: InstanceInstallStage) -> Self {
        match value {
            InstanceInstallStage::Installed => Self::Installed,
            InstanceInstallStage::Installing => Self::Installing,
            InstanceInstallStage::PackInstalling => Self::PackInstalling,
            InstanceInstallStage::NotInstalled => Self::NotInstalled,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceV1 {
    pub id: String,

    pub name: String,
    pub icon_path: Option<String>,

    pub install_stage: String,

    // Main minecraft metadata
    pub game_version: String,
    pub loader: ModLoaderV1,
    pub loader_version: Option<LoaderVersionPreferenceV1>,

    // Launch arguments
    pub java_path: Option<String>,
    pub launch_args: Option<Vec<String>>,
    pub env_vars: Option<Vec<(String, String)>>,

    // Minecraft runtime settings
    pub memory: Option<MemorySettingsV1>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSizeV1>,

    // Additional information
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    pub time_played: u64,
    pub recent_time_played: u64,

    pub hooks: HooksV1,

    pub pack_info: Option<PackInfoV1>,
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

            install_stage: {
                let dto: InstanceInstallStageV1 =
                    serde_json::from_str(&format!("\"{}\"", v1.install_stage))
                        .unwrap_or(InstanceInstallStageV1::NotInstalled);
                dto.into()
            },

            game_version: v1.game_version,
            loader: v1.loader.into(),
            loader_version: v1.loader_version.map(Into::into),

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

            memory: Overridable::new(v1.memory.unwrap_or_default().into(), v1.memory.is_some()),

            window: Overridable::new(
                WindowSettings::new(
                    v1.force_fullscreen.unwrap_or_default(),
                    v1.game_resolution.unwrap_or_default().into(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceV2 {
    pub id: String,

    pub name: String,
    pub icon_path: Option<String>,

    pub install_stage: String,

    // Metadata
    pub game_version: String,
    pub loader: ModLoaderV1,
    pub loader_version: Option<LoaderVersionPreferenceV1>,

    // Launch arguments
    pub java_path: Overridable<String>,
    pub launch_args: Overridable<Vec<String>>,
    pub env_vars: Overridable<Vec<(String, String)>>,

    // Runtime settings
    pub memory: Overridable<MemorySettingsV1>,

    // Window settings
    pub window: Overridable<WindowSettingsV1>,

    // Timestamps
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    // Stats
    pub time_played: u64,
    pub recent_time_played: u64,

    // Hooks
    pub hooks: Overridable<HooksV2>,

    pub pack_info: Option<PackInfoV2>,
}

impl From<InstanceV2> for Instance {
    fn from(v2: InstanceV2) -> Self {
        let snapshot = InstanceSnapshot {
            id: v2.id,

            name: v2.name,

            icon_path: v2.icon_path,

            install_stage: {
                let dto: InstanceInstallStageV1 =
                    serde_json::from_str(&format!("\"{}\"", v2.install_stage))
                        .unwrap_or(InstanceInstallStageV1::NotInstalled);
                dto.into()
            },

            game_version: v2.game_version,
            loader: v2.loader.into(),
            loader_version: v2.loader_version.map(Into::into),

            java_path: v2.java_path,

            launch_args: v2.launch_args,

            env_vars: v2.env_vars,

            memory: v2.memory.map(Into::into),

            window: v2.window.map(Into::into),

            created: v2.created,
            modified: v2.modified,
            last_played: v2.last_played,

            time_played: v2.time_played,
            recent_time_played: v2.recent_time_played,

            hooks: v2.hooks.map(Into::into),

            pack_info: v2.pack_info.map(Into::into),
        };

        Instance::from_snapshot(snapshot)
    }
}

impl From<&Instance> for InstanceV2 {
    fn from(value: &Instance) -> Self {
        Self {
            id: value.id().to_owned(),

            name: value.name().to_owned(),
            icon_path: value.icon_path().map(ToString::to_string),

            install_stage: {
                let dto: InstanceInstallStageV1 = value.install_stage.into();
                serde_json::to_string(&dto)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_owned()
            },

            game_version: value.game_version.clone(),
            loader: value.loader.into(),
            loader_version: value.loader_version.clone().map(Into::into),

            java_path: value.java_path.clone(),
            launch_args: value.launch_args.clone(),
            env_vars: value.env_vars.clone(),

            memory: value.memory.clone().map(Into::into),
            window: value.window.clone().map(Into::into),

            created: value.created,
            modified: value.modified,
            last_played: value.last_played,

            time_played: value.time_played,
            recent_time_played: value.recent_time_played,

            hooks: value.hooks.clone().map(Into::into),

            pack_info: value.pack_info.clone().map(Into::into),
        }
    }
}
