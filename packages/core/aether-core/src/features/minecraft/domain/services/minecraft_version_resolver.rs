use crate::features::minecraft::{MinecraftDomainError, vanilla};

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

    let is_updated = is_minecraft_updated(index, version_manifest);

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
    fn should_find_exact_version() {
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
    fn should_mark_threshold_version_as_updated() {
        let manifest = make_manifest(vec![
            make_version("1.19"),
            make_version("22w16a"),
            make_version("1.20"),
        ]);
        let (_, is_updated) = resolve_minecraft_version("22w16a", &manifest).unwrap();
        assert!(is_updated);
    }

    #[test]
    fn should_return_first_as_updated_when_threshold_not_found() {
        let manifest = make_manifest(vec![make_version("1.19"), make_version("1.20")]);
        let (_, is_updated) = resolve_minecraft_version("1.19", &manifest).unwrap();
        assert!(is_updated, "when 22w16a not found, only index 0 is updated");
    }

    #[test]
    fn should_return_not_updated_when_threshold_not_found_and_not_first() {
        let manifest = make_manifest(vec![make_version("1.19"), make_version("1.20")]);
        let (_, is_updated) = resolve_minecraft_version("1.20", &manifest).unwrap();
        assert!(
            !is_updated,
            "when 22w16a not found, index > 0 is not updated"
        );
    }

    #[test]
    fn should_find_first_match_version() {
        let manifest = make_manifest(vec![make_version("1.20"), make_version("1.20")]);
        let (version, _) = resolve_minecraft_version("1.20", &manifest).unwrap();
        assert_eq!(version.id, "1.20");
    }
}
