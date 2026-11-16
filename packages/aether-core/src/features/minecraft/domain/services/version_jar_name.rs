use crate::{
    features::minecraft::{GamePathKind, MinecraftDomainError},
    shared::safe_path::{Separators, parse_relative},
};

/// Name of the installed client jar (also its directory name) for a game version,
/// optionally combined with a mod loader version. Installed instances depend on this format.
///
/// The ids come from the metadata, and the name is a directory and a file name under the
/// versions directory: one that is not a plain name is refused.
pub fn version_jar_name(
    game_version_id: &str,
    loader_version_id: Option<&str>,
) -> Result<String, MinecraftDomainError> {
    let name = match loader_version_id {
        Some(loader) => format!("{game_version_id}-{loader}"),
        None => game_version_id.to_owned(),
    };
    ensure_plain_name(&name, GamePathKind::VersionId)?;
    Ok(name)
}

/// Refuses `name` unless it is one plain file name, for a name from the metadata that is joined
/// onto a directory.
pub fn ensure_plain_name(name: &str, kind: GamePathKind) -> Result<(), MinecraftDomainError> {
    match parse_relative(name, Separators::SlashOrBackslash) {
        Ok(path) if !path.contains('/') => Ok(()),
        _ => Err(MinecraftDomainError::UnsafeMetadataPath {
            entity_type: kind,
            value: name.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_name_is_game_version() {
        assert_eq!(version_jar_name("1.20.1", None).unwrap(), "1.20.1");
    }

    #[test]
    fn loader_name_is_joined_with_dash() {
        assert_eq!(
            version_jar_name("1.20.1", Some("forge-47.2.0")).unwrap(),
            "1.20.1-forge-47.2.0"
        );
    }

    #[test]
    fn empty_loader_id_still_appends_separator() {
        assert_eq!(version_jar_name("1.20.1", Some("")).unwrap(), "1.20.1-");
    }

    #[test]
    fn should_refuse_a_version_id_that_is_not_a_plain_name() {
        for (game, loader) in [
            ("..", None),
            ("1.20.1", Some("../..")),
            ("1.20.1", Some("a/b")),
            ("1.20.1", Some(r"a\b")),
            ("/abs", None),
            ("C:", None),
            ("", None),
        ] {
            assert!(
                matches!(
                    version_jar_name(game, loader),
                    Err(MinecraftDomainError::UnsafeMetadataPath {
                        entity_type: GamePathKind::VersionId,
                        ..
                    })
                ),
                "{game} {loader:?} was accepted"
            );
        }
    }
}
