//! Mapping of the `resolve*` handlers' inputs (core → plugin) and outputs (plugin → core).
//! [`PackSource`](crate::features::instance::PackSource) is mapped in
//! `PluginContentProviderProxy`: its host path has to be renamed into the plugin's mounts.

use aether_core_plugin_api::v0::{
    PackMetadataDto, ResolveInstallPlanParamsDto, ResolveTargetDto, ResolvedFileDto,
};

use crate::{
    features::instance::{
        InvalidContentId, PackMetadata, ResolveInstallPlanParams, ResolveTarget, ResolvedFile,
    },
    shared::hash::{FileHash, InvalidFileHash},
};

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
                game_version,
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
}

impl TryFrom<ResolvedFileDto> for ResolvedFile {
    type Error = InvalidResolvedFile;

    fn try_from(value: ResolvedFileDto) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version.try_into()?,
            url: value.url,
            hash: value.hash.map(FileHash::try_from).transpose()?,
            file_name: value.file_name,
            size: value.size,
            name: value.name,
        })
    }
}

impl TryFrom<PackMetadataDto> for PackMetadata {
    type Error = InvalidContentId;

    fn try_from(value: PackMetadataDto) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            version_id: value.version_id.map(TryInto::try_into).transpose()?,
            version_name: value.version_name,
            description: value.description,
            game_version: value.game_version,
            mod_loader: value.mod_loader.into(),
            loader_version: value.loader_version,
            // Informational; a count beyond `usize` is as good as unknown.
            file_count: value
                .file_count
                .and_then(|count| usize::try_from(count).ok()),
            icon_url: value.icon_url,
        })
    }
}
