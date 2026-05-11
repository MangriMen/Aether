use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    features::{
        minecraft::{LoaderVersionPreference, ModLoader},
        settings::{Hooks, MemorySettings, WindowSettings},
    },
    shared::Overridable,
};

use super::{InstanceInstallStage, PackInfo};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSnapshot {
    pub id: String,

    pub name: String,
    pub icon_path: Option<String>,

    pub install_stage: InstanceInstallStage,

    // Metadata
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,

    // Launch arguments
    pub java_path: Overridable<String>,
    pub launch_args: Overridable<Vec<String>>,
    pub env_vars: Overridable<Vec<(String, String)>>,

    // Runtime settings
    pub memory: Overridable<MemorySettings>,

    // Window settings
    pub window: Overridable<WindowSettings>,

    // Timestamps
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    // Stats
    pub time_played: u64,
    pub recent_time_played: u64,

    // Hooks
    pub hooks: Overridable<Hooks>,

    pub pack_info: Option<PackInfo>,
}
