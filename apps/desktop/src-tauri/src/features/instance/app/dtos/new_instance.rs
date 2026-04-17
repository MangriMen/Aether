use aether_core::features::instance::app::NewInstance;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::{
    instance::PackInfoDto,
    minecraft::{LoaderVersionPreferenceDto, ModLoaderDto},
};

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewInstanceDto {
    pub name: String,

    pub game_version: String,

    pub mod_loader: ModLoaderDto,

    #[specta(optional)]
    pub loader_version: Option<LoaderVersionPreferenceDto>,

    #[specta(optional)]
    pub icon_path: Option<String>,

    #[specta(optional)]
    pub skip_install_instance: Option<bool>,

    #[specta(optional)]
    pub pack_info: Option<PackInfoDto>,
}

impl From<NewInstanceDto> for NewInstance {
    fn from(value: NewInstanceDto) -> Self {
        Self {
            name: value.name,
            game_version: value.game_version,
            mod_loader: value.mod_loader.into(),
            loader_version: value.loader_version.map(|x| x.into()),
            icon_path: value.icon_path,
            skip_install_instance: value.skip_install_instance,
            pack_info: value.pack_info.map(|x| x.into()),
        }
    }
}
