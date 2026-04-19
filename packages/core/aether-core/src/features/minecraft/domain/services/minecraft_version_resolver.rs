use crate::features::minecraft::{MinecraftDomainError, vanilla};

pub fn resolve_minecraft_version(
    game_version: &str,
    version_manifest: vanilla::VersionManifest,
) -> Result<(vanilla::Version, bool), MinecraftDomainError> {
    let (index, version) = version_manifest
        .versions
        .iter()
        .enumerate()
        .find(|(_, v)| v.id == game_version)
        .ok_or(MinecraftDomainError::VersionNotFound {
            version: game_version.to_owned(),
        })?;

    let is_updated = is_minecraft_updated(index, &version_manifest);

    Ok((version.clone(), is_updated))
}

fn is_minecraft_updated(version_index: usize, version_manifest: &vanilla::VersionManifest) -> bool {
    version_index
        <= version_manifest
            .versions
            .iter()
            .position(|version| version.id == "22w16a")
            .unwrap_or(0)
}
