use crate::features::minecraft::{MinecraftDomainError, vanilla};

/// 2022-04-20T00:00:00Z, the day 22w16a came out: the first version that runs natively on every
/// desktop OS the launcher supports.
const NATIVE_SUPPORT_SINCE: i64 = 1_650_412_800;

pub fn resolve_minecraft_version(
    game_version: &str,
    version_manifest: &vanilla::VersionManifest,
) -> Result<(vanilla::Version, bool), MinecraftDomainError> {
    let (index, version) = version_manifest
        .versions
        .iter()
        .enumerate()
        .find(|(_, v)| v.id == game_version)
        .ok_or(MinecraftDomainError::VersionNotFound {
            version: game_version.to_owned(),
        })?;

    let is_updated = is_minecraft_updated(index, version, version_manifest);

    Ok((version.clone(), is_updated))
}

/// The manifest lists the newest version first, so a version is at or after 22w16a when it sits
/// no later than it. A manifest without 22w16a falls back to the release time instead.
fn is_minecraft_updated(
    version_index: usize,
    version: &vanilla::Version,
    version_manifest: &vanilla::VersionManifest,
) -> bool {
    match version_manifest
        .versions
        .iter()
        .position(|version| version.id == "22w16a")
    {
        Some(anchor) => version_index <= anchor,
        None => version.release_time.timestamp() >= NATIVE_SUPPORT_SINCE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::minecraft::vanilla;

    fn make_version(id: &str) -> vanilla::Version {
        vanilla::Version {
            id: id.to_string(),
            type_: vanilla::VersionType::Release,
            url: String::new(),
            time: chrono::DateTime::UNIX_EPOCH,
            release_time: chrono::DateTime::UNIX_EPOCH,
            sha1: String::new(),
            compliance_level: 0,
            original_sha1: None,
        }
    }

    fn make_manifest(versions: Vec<vanilla::Version>) -> vanilla::VersionManifest {
        vanilla::VersionManifest {
            latest: vanilla::LatestVersion {
                release: "1.20".into(),
                snapshot: "24w10a".into(),
            },
            versions,
        }
    }

    #[test]
    fn should_find_the_exact_version_when_it_is_in_the_list() {
        let manifest = make_manifest(vec![
            make_version("1.19"),
            make_version("1.20"),
            make_version("1.21"),
        ]);
        let (version, _) = resolve_minecraft_version("1.20", &manifest).unwrap();
        assert_eq!(version.id, "1.20");
    }

    #[test]
    fn should_return_error_when_version_not_found() {
        let manifest = make_manifest(vec![make_version("1.19")]);
        let err = resolve_minecraft_version("1.20", &manifest).unwrap_err();
        assert!(matches!(err, MinecraftDomainError::VersionNotFound { .. }));
    }

    #[test]
    fn should_mark_as_updated_when_below_threshold() {
        let manifest = make_manifest(vec![
            make_version("1.19"),
            make_version("22w16a"),
            make_version("1.20"),
        ]);
        let (_, is_updated) = resolve_minecraft_version("1.19", &manifest).unwrap();
        assert!(is_updated);
    }

    #[test]
    fn should_mark_as_not_updated_when_above_threshold() {
        let manifest = make_manifest(vec![
            make_version("22w16a"),
            make_version("1.20"),
            make_version("1.21"),
        ]);
        let (_, is_updated) = resolve_minecraft_version("1.21", &manifest).unwrap();
        assert!(!is_updated);
    }

    #[test]
    fn should_mark_the_version_as_updated_when_it_is_the_threshold() {
        let manifest = make_manifest(vec![
            make_version("1.19"),
            make_version("22w16a"),
            make_version("1.20"),
        ]);
        let (_, is_updated) = resolve_minecraft_version("22w16a", &manifest).unwrap();
        assert!(is_updated);
    }

    fn released_at(id: &str, timestamp: i64) -> vanilla::Version {
        vanilla::Version {
            release_time: chrono::DateTime::from_timestamp(timestamp, 0).unwrap(),
            ..make_version(id)
        }
    }

    #[test]
    fn should_mark_as_updated_by_release_time_when_threshold_not_found_and_version_is_newer() {
        let manifest = make_manifest(vec![
            released_at("1.20", NATIVE_SUPPORT_SINCE + 1),
            released_at("1.19", NATIVE_SUPPORT_SINCE),
        ]);
        let (_, is_updated) = resolve_minecraft_version("1.19", &manifest).unwrap();
        assert!(is_updated);
    }

    #[test]
    fn should_mark_as_not_updated_by_release_time_when_threshold_not_found_and_version_is_older() {
        let manifest = make_manifest(vec![
            released_at("1.20", NATIVE_SUPPORT_SINCE + 1),
            released_at("1.18", NATIVE_SUPPORT_SINCE - 1),
        ]);
        let (_, is_updated) = resolve_minecraft_version("1.18", &manifest).unwrap();
        assert!(!is_updated);
    }

    #[test]
    fn should_find_the_first_match_when_several_versions_match() {
        let manifest = make_manifest(vec![make_version("1.20"), make_version("1.20")]);
        let (version, _) = resolve_minecraft_version("1.20", &manifest).unwrap();
        assert_eq!(version.id, "1.20");
    }
}
