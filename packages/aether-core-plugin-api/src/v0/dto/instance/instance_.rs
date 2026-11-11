use serde::{Deserialize, Serialize};

use crate::v0::{InstanceInstallStageDto, LoaderVersionPreferenceDto, ModLoaderDto, PackInfoDto};

/// What a plugin may learn about an instance it was not asked to work on: enough to judge
/// whether content fits it.
///
/// Deliberately without launch settings (`java_path`, `launch_args`, `env_vars`, `hooks`) and
/// host paths: users keep API keys and proxy credentials in the environment variables, and a
/// plugin with network access could send them out. Add a field here only when every plugin that
/// receives it needs it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSummaryDto {
    pub id: String,

    pub name: String,

    pub install_stage: InstanceInstallStageDto,

    pub game_version: String,
    pub loader: ModLoaderDto,
    pub loader_version: Option<LoaderVersionPreferenceDto>,

    pub pack_info: Option<PackInfoDto>,
}
