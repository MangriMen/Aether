use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::v0::{
    HooksDto, InstanceInstallStageDto, LoaderVersionPreferenceDto, MemorySettingsDto, ModLoaderDto,
    OverridableDto, PackInfoDto, WindowSettingsDto,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub time_played: u64,
    pub recent_time_played: u64,

    // Hooks
    pub hooks: OverridableDto<HooksDto>,

    pub pack_info: Option<PackInfoDto>,
}
