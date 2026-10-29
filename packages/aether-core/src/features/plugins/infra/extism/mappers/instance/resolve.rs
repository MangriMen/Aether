//! Mapping of the `resolve*` handlers' inputs (core → plugin) and outputs (plugin → core).
//! [`PackSource`](crate::features::instance::PackSource) is mapped in
//! `PluginContentProviderProxy`: its host path has to be renamed into the plugin's mounts.

use aether_core_plugin_api::v0::{
    PackMetadataDto, ResolveInstallPlanParamsDto, ResolveTargetDto, ResolvedFileDto,
};

use crate::{
    features::instance::{PackMetadata, ResolveInstallPlanParams, ResolveTarget, ResolvedFile},
    shared::hash::{FileHash, InvalidFileHash},
};

impl From<ResolveInstallPlanParams> for ResolveInstallPlanParamsDto {
    fn from(value: ResolveInstallPlanParams) -> Self {
        Self {
            content_id: value.content_id,
            version: value.version,
            installed_version: value.installed_version,
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
                loader,
            },
            ResolveTarget::New => Self::New,
        }
    }
}

impl TryFrom<ResolvedFileDto> for ResolvedFile {
    type Error = InvalidFileHash;

    fn try_from(value: ResolvedFileDto) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version,
            url: value.url,
            hash: value.hash.map(FileHash::try_from).transpose()?,
            file_name: value.file_name,
            size: value.size,
            name: value.name,
        })
    }
}

impl From<PackMetadataDto> for PackMetadata {
    fn from(value: PackMetadataDto) -> Self {
        Self {
            name: value.name,
            version_id: value.version_id,
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
        }
    }
}
