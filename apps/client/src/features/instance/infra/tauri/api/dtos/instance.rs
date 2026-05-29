use aether_core::features::instance::Instance;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use specta::Type;

use crate::{
    core::{OverridableDto, format_asset_url},
    features::{
        instance::infra::{InstanceInstallStageDto, PackInfoDto},
        minecraft::infra::{LoaderVersionPreferenceDto, ModLoaderDto},
        settings::infra::{
            dtos::WindowSettingsDto,
            dtos::{HooksDto, MemorySettingsDto},
        },
    },
};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct InstanceDto {
    pub id: String,

    pub name: String,
    pub icon_path: Option<String>,

    pub install_stage: InstanceInstallStageDto,

    // Metadata
    pub game_version: String,
    pub loader: ModLoaderDto,
    pub loader_version: Option<LoaderVersionPreferenceDto>,

    // Launch arguments
    pub java_path: OverridableDto<String>,
    pub launch_args: OverridableDto<Vec<String>>,
    pub env_vars: OverridableDto<Vec<(String, String)>>,

    // Runtime settings
    pub memory: OverridableDto<MemorySettingsDto>,

    // Window settings
    pub window: OverridableDto<WindowSettingsDto>,

    // Timestamps
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    // Stats
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub time_played: u64,
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub recent_time_played: u64,

    // Hooks
    pub hooks: OverridableDto<HooksDto>,

    pub pack_info: Option<PackInfoDto>,
}

impl From<Instance> for InstanceDto {
    fn from(value: Instance) -> Self {
        Self {
            id: value.id().to_owned(),

            name: value.name().to_owned(),

            icon_path: value.icon_path().map(format_asset_url),

            install_stage: value.install_stage.into(),

            game_version: value.game_version,
            loader: value.loader.into(),
            loader_version: value.loader_version.map(Into::into),

            java_path: value.java_path.into(),
            launch_args: value.launch_args.into(),
            env_vars: value.env_vars.into(),

            memory: value.memory.into(),

            window: value.window.into(),

            created: value.created,
            modified: value.modified,
            last_played: value.last_played,

            time_played: value.time_played,
            recent_time_played: value.recent_time_played,

            hooks: value.hooks.into(),

            pack_info: value.pack_info.map(Into::into),
        }
    }
}
