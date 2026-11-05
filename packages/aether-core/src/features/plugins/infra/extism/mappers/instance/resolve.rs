//! Mapping of the `resolve*` handlers' inputs (core → plugin) and outputs (plugin → core).
//! [`PackSource`](crate::features::instance::PackSource) is mapped in
//! `PluginContentProviderProxy`: its host path has to be renamed into the plugin's mounts.

use aether_core_plugin_api::v0::{
    PackMetadataDto, ResolveInstallPlanParamsDto, ResolveTargetDto, ResolvedFileDto,
};

use crate::{
    features::{
        instance::{
            InvalidContentId, PackMetadata, ResolveInstallPlanParams, ResolveTarget, ResolvedFile,
        },
        minecraft::{GameVersion, InvalidGameVersion},
    },
    shared::{
        hash::{FileHash, InvalidFileHash},
        web_url::{HttpsUrl, InvalidUrl},
    },
};

use crate::shared::web_url::display_url;

impl From<ResolveInstallPlanParams> for ResolveInstallPlanParamsDto {
    fn from(value: ResolveInstallPlanParams) -> Self {
        Self {
            content_id: value.content_id.into(),
            version: value.version.map(Into::into),
            installed_version: value.installed_version.map(Into::into),
            target: value.target.into(),
        }
    }
}

impl From<ResolveTarget> for ResolveTargetDto {
    fn from(value: ResolveTarget) -> Self {
        match value {
            ResolveTarget::Existing {
                instance_id,
                game_version,
                loader,
            } => Self::Existing {
                instance_id: instance_id.to_string(),
                game_version: game_version.into(),
                loader: loader.map(|loader| loader.as_str().to_owned()),
            },
            ResolveTarget::New => Self::New,
        }
    }
}

/// What is wrong with a file a plugin resolved a ref to.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidResolvedFile {
    #[error(transparent)]
    Hash(#[from] InvalidFileHash),

    #[error(transparent)]
    ContentId(#[from] InvalidContentId),

    #[error(transparent)]
    Url(#[from] InvalidUrl),
}

impl TryFrom<ResolvedFileDto> for ResolvedFile {
    type Error = InvalidResolvedFile;

    fn try_from(value: ResolvedFileDto) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version.try_into()?,
            url: HttpsUrl::try_from(value.url)?,
            hash: value.hash.map(FileHash::try_from).transpose()?,
            file_name: value.file_name,
            size: value.size,
            name: value.name,
        })
    }
}

/// What is wrong with the description of a pack a plugin returned.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidPackMetadata {
    #[error(transparent)]
    ContentId(#[from] InvalidContentId),

    #[error(transparent)]
    GameVersion(#[from] InvalidGameVersion),
}

impl TryFrom<PackMetadataDto> for PackMetadata {
    type Error = InvalidPackMetadata;

    fn try_from(value: PackMetadataDto) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            version_id: value.version_id.map(TryInto::try_into).transpose()?,
            version_name: value.version_name,
            description: value.description,
            game_version: GameVersion::try_from(value.game_version)?,
            mod_loader: value.mod_loader.into(),
            loader_version: value.loader_version,
            // Informational; a count beyond `usize` is as good as unknown.
            file_count: value
                .file_count
                .and_then(|count| usize::try_from(count).ok()),
            icon_url: display_url("icon of a plugin's pack", value.icon_url),
        })
    }
}

#[cfg(test)]
mod tests {
    use aether_core_plugin_api::v0::ModLoaderDto;

    use super::*;

    fn pack(game_version: &str) -> PackMetadataDto {
        PackMetadataDto {
            name: "Pack".into(),
            version_id: None,
            version_name: None,
            description: None,
            game_version: game_version.into(),
            mod_loader: ModLoaderDto::Fabric,
            loader_version: None,
            file_count: None,
            icon_url: None,
        }
    }

    #[test]
    fn should_refuse_the_pack_when_its_game_version_is_not_a_valid_id() {
        let err = PackMetadata::try_from(pack("1.21/../..")).unwrap_err();

        assert!(matches!(err, InvalidPackMetadata::GameVersion(_)), "{err}");
    }

    #[test]
    fn should_keep_the_game_version_of_the_pack_when_it_is_valid() {
        let metadata = PackMetadata::try_from(pack("1.14 Pre-Release 1")).unwrap();

        assert_eq!(metadata.game_version, "1.14 Pre-Release 1");
    }
}
