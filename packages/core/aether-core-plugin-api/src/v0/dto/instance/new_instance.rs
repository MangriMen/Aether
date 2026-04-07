use serde::{Deserialize, Serialize};

use crate::v0::{LoaderVersionPreferenceDto, ModLoaderDto, PackInfoDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInstanceDto {
    pub name: String,
    pub game_version: String,
    pub mod_loader: ModLoaderDto,
    pub loader_version: Option<LoaderVersionPreferenceDto>,
    pub icon_path: Option<String>,
    pub skip_install_instance: Option<bool>,
    pub pack_info: Option<PackInfoDto>,
}
