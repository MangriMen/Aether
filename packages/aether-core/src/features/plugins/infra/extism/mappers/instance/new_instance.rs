use aether_core_plugin_api::v0::NewInstanceDto;

use crate::features::instance::{InvalidId, NewInstance};

impl TryFrom<NewInstanceDto> for NewInstance {
    type Error = InvalidId;

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
