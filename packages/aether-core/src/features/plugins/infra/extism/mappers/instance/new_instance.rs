use aether_core_plugin_api::v0::NewInstanceDto;

use super::pack_info::InvalidPackInfo;
use crate::features::{
    instance::{ContentProviderRejection, InstanceError, InstanceName, NewInstance},
    minecraft::GameVersion,
};

impl From<InvalidPackInfo> for ContentProviderRejection {
    fn from(err: InvalidPackInfo) -> Self {
        match err {
            InvalidPackInfo::Provider(err) => err.into(),
            InvalidPackInfo::ContentId(err) => err.into(),
        }
    }
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
            pack_info: value
                .pack_info
                .map(TryInto::try_into)
                .transpose()
                .map_err(|err: InvalidPackInfo| ContentProviderRejection::from(err))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::capability::InvalidId;

    #[test]
    fn should_refuse_a_pack_info_with_an_invalid_provider_id_as_an_invalid_id_of_the_answer() {
        let reason = ContentProviderRejection::from(InvalidPackInfo::Provider(InvalidId {
            kind: "plugin",
            value: "Bad".to_owned(),
        }));

        assert!(
            matches!(
                reason,
                ContentProviderRejection::ResponseInvalidId { kind: "plugin", ref value }
                    if value == "Bad"
            ),
            "{reason:?}"
        );
    }
}
