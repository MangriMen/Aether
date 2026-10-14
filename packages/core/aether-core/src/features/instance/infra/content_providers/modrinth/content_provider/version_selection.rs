use chrono::{DateTime, Utc};

use crate::features::instance::{
    ChannelVersion, ContentType, ContentVersionType, Instance, ReleaseChannel,
    infra::content_providers::modrinth::api_client::ProjectResponse, select_version,
};

use super::super::api_client::{File, ProjectVersionResponse};

pub(super) fn get_first_file_from_project_version(
    version: &ProjectVersionResponse,
) -> Option<&File> {
    version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
}

/// The version of a project to install into an instance running `game_version` with `loader`,
/// by the release channel ([`select_version`]) from the version `installed` now, if any.
pub(super) fn find_best_version<'a>(
    response: &'a [ProjectVersionResponse],
    game_version: &str,
    loader: Option<&str>,
    installed: Option<&'a ProjectVersionResponse>,
) -> Option<&'a ProjectVersionResponse> {
    let fitting: Vec<&ProjectVersionResponse> = response
        .iter()
        .filter(|v| {
            let is_right_game_version = v.game_versions.iter().any(|gv| gv == game_version);
            let is_right_loader = loader.is_none_or(|l| v.loaders.iter().any(|vl| vl == l));

            is_right_game_version && is_right_loader
        })
        .collect();

    select_version(&fitting, installed.as_ref(), ReleaseChannel::default()).copied()
}

impl ChannelVersion for ProjectVersionResponse {
    fn id(&self) -> &str {
        &self.id
    }

    fn version_type(&self) -> ContentVersionType {
        self.version_type.clone().into()
    }

    fn date_published(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.date_published)
            .ok()
            .map(|date| date.to_utc())
    }
}

pub(super) fn is_version_compatible(
    version: &ProjectVersionResponse,
    project: &ProjectResponse,
    instance: &Instance,
) -> bool {
    let is_game_version_compatible = version.game_versions.contains(&instance.game_version);

    if !is_game_version_compatible {
        return false;
    }

    if project.project_type == ContentType::Mod.to_str() {
        version
            .loaders
            .iter()
            .any(|l| l == instance.loader.as_str() || l == ContentType::DataPack.to_str())
    } else {
        true
    }
}
