use std::cmp::Ordering;

use crate::features::{
    instance::{
        infra::content_providers::modrinth::api_client::{
            ModrinthIndex, ModrinthVersionType, ProjectResponse,
        },
        ContentType, Instance,
    },
    minecraft::ModLoader,
};

use super::super::api_client::{File, ProjectVersionResponse};

pub fn get_facet_vector<T: AsRef<str>>(facet: &str, values: &[T]) -> Vec<String> {
    values
        .iter()
        .map(|value| format!("{}:{}", facet, value.as_ref()))
        .collect()
}

pub fn get_first_file_from_project_version(version: &ProjectVersionResponse) -> Option<&File> {
    version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
}

pub fn find_best_version<'a>(
    response: &'a [ProjectVersionResponse],
    game_version: &str,
    loader: &Option<String>,
) -> Option<&'a ProjectVersionResponse> {
    response
        .iter()
        .filter(|v| {
            let is_right_game_version = v.game_versions.iter().any(|gv| gv == game_version);
            let is_right_loader = loader
                .as_deref()
                .map(|l| v.loaders.iter().any(|vl| vl == l))
                .unwrap_or(true);

            is_right_game_version && is_right_loader
        })
        .max_by(|a, b| compare_versions(a, b))
}

pub fn compare_versions(a: &ProjectVersionResponse, b: &ProjectVersionResponse) -> Ordering {
    let stability_priority = |version_type: &ModrinthVersionType| match version_type {
        ModrinthVersionType::Release => 3,
        ModrinthVersionType::Beta => 2,
        ModrinthVersionType::Alpha => 1,
    };

    let a_stability = stability_priority(&a.version_type);
    let b_stability = stability_priority(&b.version_type);

    // Compare by stability first, then by publication date if stability is the same
    a_stability
        .cmp(&b_stability)
        .then_with(|| a.date_published.cmp(&b.date_published))
}

pub fn resolve_loader_from_manifest(manifest: &ModrinthIndex) -> (ModLoader, Option<String>) {
    let loaders = &manifest.dependencies.loaders;

    if let Some(v) = loaders.get("fabric-loader") {
        return (ModLoader::Fabric, Some(v.clone()));
    }
    if let Some(v) = loaders.get("forge") {
        return (ModLoader::Forge, Some(v.clone()));
    }
    if let Some(v) = loaders.get("quilt-loader") {
        return (ModLoader::Quilt, Some(v.clone()));
    }

    (ModLoader::Vanilla, None)
}

pub fn is_version_compatible(
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
