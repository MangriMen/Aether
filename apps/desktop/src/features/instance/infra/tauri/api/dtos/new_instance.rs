use aether_core::features::{
    instance::{InstanceError, InstanceName, NewInstance},
    minecraft::GameVersion,
};
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
            name: InstanceName::try_from(value.name)?,
            game_version: GameVersion::try_from(value.game_version)?,
            mod_loader: value.mod_loader.into(),
            loader_version: value.loader_version.map(TryInto::try_into).transpose()?,
            icon_path: value.icon_path,
            skip_install_instance: value.skip_install_instance,
            pack_info: value.pack_info.map(TryInto::try_into).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dto(game_version: &str) -> NewInstanceDto {
        NewInstanceDto {
            name: "Pack".into(),
            game_version: game_version.into(),
            mod_loader: ModLoaderDto::Vanilla,
            loader_version: None,
            icon_path: None,
            skip_install_instance: None,
            pack_info: None,
        }
    }

    #[test]
    fn should_refuse_a_new_instance_when_its_game_version_is_not_a_valid_id() {
        for bad in ["", " 1.21", "1.21/..", "..", "1.21\n"] {
            let result = NewInstance::try_from(dto(bad));

            assert!(
                matches!(result, Err(InstanceError::InvalidGameVersion { .. })),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn should_keep_the_game_version_of_a_new_instance_when_it_is_valid() {
        let new_instance = NewInstance::try_from(dto("1.14 Pre-Release 1")).unwrap();

        assert_eq!(new_instance.game_version, "1.14 Pre-Release 1");
    }

    #[test]
    fn should_refuse_a_new_instance_when_its_loader_version_is_not_a_valid_id() {
        for bad in ["", " 0.16.5", "0.16 .5", "0.16.5\n"] {
            let mut dto = dto("1.21.1");
            dto.mod_loader = ModLoaderDto::Fabric;
            dto.loader_version = Some(LoaderVersionPreferenceDto::Exact(bad.into()));

            let result = NewInstance::try_from(dto);

            assert!(
                matches!(result, Err(InstanceError::InvalidLoaderVersion { .. })),
                "{bad:?}"
            );
        }
    }
}
