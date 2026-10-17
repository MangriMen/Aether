/// Name of the installed client jar (also its directory name) for a game version,
/// optionally combined with a mod loader version. Installed instances depend on this format.
pub fn version_jar_name(game_version_id: &str, loader_version_id: Option<&str>) -> String {
    match loader_version_id {
        Some(loader) => format!("{game_version_id}-{loader}"),
        None => game_version_id.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_name_is_game_version() {
        assert_eq!(version_jar_name("1.20.1", None), "1.20.1");
    }

    #[test]
    fn loader_name_is_joined_with_dash() {
        assert_eq!(
            version_jar_name("1.20.1", Some("forge-47.2.0")),
            "1.20.1-forge-47.2.0"
        );
    }

    #[test]
    fn empty_loader_id_still_appends_separator() {
        assert_eq!(version_jar_name("1.20.1", Some("")), "1.20.1-");
    }
}
