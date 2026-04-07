use aether_core_plugin_api::v0::NewInstanceDto;

use crate::features::instance::app::NewInstance;

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
