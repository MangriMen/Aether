use aether_core::features::instance::{InstanceError, NewInstance};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::{
    instance::infra::PackInfoDto,
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

impl TryFrom<NewInstanceDto> for NewInstance {
    type Error = InstanceError;

    fn try_from(value: NewInstanceDto) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            game_version: value.game_version,
            mod_loader: value.mod_loader.into(),
            loader_version: value.loader_version.map(std::convert::Into::into),
            icon_path: value.icon_path,
            skip_install_instance: value.skip_install_instance,
            pack_info: value.pack_info.map(TryInto::try_into).transpose()?,
        })
    }
}
