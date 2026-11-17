use aether_core_plugin_api::v0::NewInstanceDto;

use super::{UnknownValue, pack_info::InvalidPackInfo};
use crate::{
    features::{
        instance::{ContentProviderRejection, InstanceError, InstanceName, NewInstance},
        minecraft::{GameVersion, LoaderVersionPreference},
    },
    shared::error::shared_source,
};

impl From<InvalidPackInfo> for ContentProviderRejection {
    fn from(err: InvalidPackInfo) -> Self {
        match err {
            InvalidPackInfo::Provider(err) => err.into(),
            InvalidPackInfo::ContentId(err) => err.into(),
        }
    }
}

/// What is wrong with an instance a plugin described: a new one, or the game of an existing one.
#[derive(Debug, Clone, thiserror::Error)]
pub enum InvalidInstanceDescription {
    /// A value the core checks the same way wherever it comes from: the name, an id, a version.
    #[error(transparent)]
    Instance(#[from] InstanceError),

    /// A value of an enum the plugin's API version does not have.
    #[error(transparent)]
    Value(#[from] UnknownValue),
}

/// As the argument of a host function: a value the plugin's API version does not have makes the
/// request invalid.
impl From<InvalidInstanceDescription> for InstanceError {
    fn from(err: InvalidInstanceDescription) -> Self {
        match err {
            InvalidInstanceDescription::Instance(err) => err,
            InvalidInstanceDescription::Value(err) => {
                ContentProviderRejection::InvalidRequest(shared_source(err)).into()
            }
        }
    }
}

impl TryFrom<NewInstanceDto> for NewInstance {
    type Error = InvalidInstanceDescription;

    fn try_from(value: NewInstanceDto) -> Result<Self, Self::Error> {
        Ok(Self {
            name: InstanceName::try_from(value.name)?,
            game_version: GameVersion::try_from(value.game_version).map_err(InstanceError::from)?,
            mod_loader: value.mod_loader.try_into()?,
            loader_version: value
                .loader_version
                .map(LoaderVersionPreference::try_from)
                .transpose()
                .map_err(InstanceError::from)?,
            icon_path: value.icon_path,
            skip_install_instance: value.skip_install_instance,
            pack_info: value.pack_info.map(TryInto::try_into).transpose().map_err(
                |err: InvalidPackInfo| InstanceError::from(ContentProviderRejection::from(err)),
            )?,
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

#[cfg(test)]
mod unknown_loader_tests {
    use aether_core_plugin_api::v0::ModLoaderDto;

    use super::*;

    #[test]
    fn should_refuse_a_new_instance_when_its_loader_is_unknown() {
        let err = NewInstance::try_from(NewInstanceDto {
            name: "Pack".into(),
            game_version: "1.21.1".into(),
            mod_loader: ModLoaderDto::Unknown,
            loader_version: None,
            icon_path: None,
            skip_install_instance: None,
            pack_info: None,
        })
        .unwrap_err();

        assert!(
            matches!(
                err,
                InvalidInstanceDescription::Value(UnknownValue {
                    kind: "mod loader",
                    ..
                })
            ),
            "{err}"
        );
    }
}
