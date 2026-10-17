use std::path::Path;

use uuid::Uuid;

use crate::{
    features::{
        auth::Credential,
        minecraft::{MinecraftDomainError, parse_arguments, vanilla},
        settings::WindowSize,
    },
    shared::io::infra::canonicalize,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn get_minecraft_arguments(
    arguments: Option<&[vanilla::Argument]>,
    legacy_arguments: Option<&str>,
    credentials: &Credential,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &vanilla::VersionType,
    resolution: WindowSize,
    java_arch: &str,
) -> Result<Vec<String>, MinecraftDomainError> {
    let substitute = |template: &str| {
        replace_placeholders_in_argument_string(
            template,
            credentials.access_token(),
            credentials.username().as_str(),
            credentials.id(),
            version,
            asset_index_name,
            game_directory,
            assets_directory,
            version_type,
            resolution,
        )
    };

    if let Some(arguments) = arguments {
        let mut result = Vec::new();
        parse_arguments(arguments, &mut result, substitute, java_arch)?;
        Ok(result)
    } else if let Some(legacy) = legacy_arguments {
        legacy.split(' ').map(substitute).collect()
    } else {
        Ok(Vec::new())
    }
}

fn canonical_dir(path: &Path, entity_type: &str) -> Result<String, MinecraftDomainError> {
    canonicalize(path)
        .map(|canonical| canonical.to_string_lossy().into_owned())
        .map_err(|_| MinecraftDomainError::PathNotFound {
            path: path.to_path_buf(),
            entity_type: entity_type.to_owned(),
        })
}

#[allow(clippy::too_many_arguments)]
fn replace_placeholders_in_argument_string(
    argument: &str,
    access_token: &str,
    username: &str,
    uuid: Uuid,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &vanilla::VersionType,
    resolution: WindowSize,
) -> Result<String, MinecraftDomainError> {
    let game = canonical_dir(game_directory, "game")?;
    let assets = canonical_dir(assets_directory, "assets")?;
    let uuid = uuid.simple().to_string();
    let width = resolution.width().to_string();
    let height = resolution.height().to_string();

    let table = [
        ("${accessToken}", access_token),
        ("${auth_access_token}", access_token),
        ("${auth_session}", access_token),
        ("${auth_player_name}", username),
        ("${auth_xuid}", "0"),
        ("${auth_uuid}", uuid.as_str()),
        ("${uuid}", uuid.as_str()),
        ("${clientid}", "c4502edb-87c6-40cb-b595-64a280cf8906"),
        ("${user_properties}", "{}"),
        ("${user_type}", "msa"),
        ("${version_name}", version),
        ("${assets_index_name}", asset_index_name),
        ("${game_directory}", game.as_str()),
        ("${assets_root}", assets.as_str()),
        ("${game_assets}", assets.as_str()),
        ("${version_type}", version_type.as_str()),
        ("${resolution_width}", width.as_str()),
        ("${resolution_height}", height.as_str()),
    ];

    Ok(table
        .iter()
        .fold(argument.to_owned(), |acc, (name, value)| {
            acc.replace(name, value)
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{
        auth::{AccountType, Username},
        minecraft::vanilla::{
            Argument, ArgumentValue, FeatureRule, Os, OsRule, Rule, RuleAction, VersionType,
        },
    };
    use chrono::Utc;

    const UUID_STR: &str = "0123456789abcdef0123456789abcdef";

    struct Env {
        _tmp: tempfile::TempDir,
        game: std::path::PathBuf,
        assets: std::path::PathBuf,
    }

    fn env() -> Env {
        let tmp = tempfile::tempdir().unwrap();
        let game = tmp.path().join("game dir");
        let assets = tmp.path().join("assets");
        std::fs::create_dir_all(&game).unwrap();
        std::fs::create_dir_all(&assets).unwrap();
        Env {
            _tmp: tmp,
            game,
            assets,
        }
    }

    fn canon(p: &Path) -> String {
        canonicalize(p).unwrap().to_string_lossy().to_string()
    }

    fn credential() -> Credential {
        Credential::new(
            Uuid::parse_str(UUID_STR).unwrap(),
            Username::parse("Steve_1").unwrap(),
            AccountType::Microsoft,
            true,
            "tok-123".to_string(),
            "refresh".to_string(),
            Utc::now(),
        )
    }

    fn call(
        e: &Env,
        arguments: Option<&[Argument]>,
        legacy: Option<&str>,
    ) -> Result<Vec<String>, MinecraftDomainError> {
        get_minecraft_arguments(
            arguments,
            legacy,
            &credential(),
            "1.20.1",
            "12",
            &e.game,
            &e.assets,
            &VersionType::Release,
            WindowSize::new(1280, 720),
            "x64",
        )
    }

    fn normal(s: &str) -> Argument {
        Argument::Normal(s.to_string())
    }

    fn allow() -> Vec<Rule> {
        vec![Rule {
            action: RuleAction::Allow,
            os: Some(OsRule {
                name: Some(Os::native()),
                version: None,
                arch: None,
            }),
            features: None,
        }]
    }

    fn deny() -> Vec<Rule> {
        vec![Rule {
            action: RuleAction::Allow,
            os: Some(OsRule {
                name: Some(Os::Unknown),
                version: None,
                arch: None,
            }),
            features: None,
        }]
    }

    #[test]
    fn no_arguments_and_no_legacy_gives_empty_without_touching_filesystem() {
        let e = Env {
            _tmp: tempfile::tempdir().unwrap(),
            game: "/definitely/missing/game".into(),
            assets: "/definitely/missing/assets".into(),
        };
        assert!(call(&e, None, None).unwrap().is_empty());
    }

    #[test]
    fn modern_arguments_replace_all_placeholders() {
        let e = env();
        let args = vec![
            normal("--username ${auth_player_name}"),
            normal("--version ${version_name}"),
            normal("--gameDir ${game_directory}"),
            normal("--assetsDir ${assets_root}"),
            normal("--assetIndex ${assets_index_name}"),
            normal("--uuid ${auth_uuid}"),
            normal("--accessToken ${auth_access_token}"),
            normal("--clientId ${clientid}"),
            normal("--xuid ${auth_xuid}"),
            normal("--userType ${user_type}"),
            normal("--versionType ${version_type}"),
            normal("--userProperties ${user_properties}"),
            normal("--width ${resolution_width} --height ${resolution_height}"),
        ];
        let out = call(&e, Some(&args), None).unwrap();
        assert_eq!(
            out,
            vec![
                "--username".to_string(),
                "Steve_1".to_string(),
                "--version".to_string(),
                "1.20.1".to_string(),
                "--gameDir".to_string(),
                canon(&e.game),
                "--assetsDir".to_string(),
                canon(&e.assets),
                "--assetIndex".to_string(),
                "12".to_string(),
                "--uuid".to_string(),
                UUID_STR.to_string(),
                "--accessToken".to_string(),
                "tok-123".to_string(),
                "--clientId".to_string(),
                "c4502edb-87c6-40cb-b595-64a280cf8906".to_string(),
                "--xuid".to_string(),
                "0".to_string(),
                "--userType".to_string(),
                "msa".to_string(),
                "--versionType".to_string(),
                "release".to_string(),
                "--userProperties".to_string(),
                "{}".to_string(),
                "--width".to_string(),
                "1280".to_string(),
                "--height".to_string(),
                "720".to_string(),
            ]
        );
    }

    #[test]
    fn alias_placeholders_resolve_to_same_values() {
        let e = env();
        let args = vec![
            normal("${accessToken}"),
            normal("${auth_session}"),
            normal("${uuid}"),
            normal("${game_assets}"),
        ];
        let out = call(&e, Some(&args), None).unwrap();
        assert_eq!(
            out,
            vec![
                "tok-123".to_string(),
                "tok-123".to_string(),
                UUID_STR.to_string(),
                canon(&e.assets),
            ]
        );
    }

    #[test]
    fn multiple_placeholders_in_one_argument_are_replaced() {
        let e = env();
        let args = vec![normal(
            "${auth_player_name}@${version_name}/${version_name}",
        )];
        let out = call(&e, Some(&args), None).unwrap();
        assert_eq!(out, vec!["Steve_1@1.20.1/1.20.1"]);
    }

    #[test]
    fn game_directory_with_spaces_is_not_split() {
        let e = env();
        let out = call(&e, Some(&[normal("--gameDir ${game_directory}")]), None).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[1], canon(&e.game));
        assert!(out[1].contains("game dir"));
    }

    #[test]
    fn unknown_placeholders_are_left_untouched() {
        let e = env();
        let out = call(&e, Some(&[normal("--x ${nope}")]), None).unwrap();
        assert_eq!(out, vec!["--x", "${nope}"]);
    }

    #[test]
    fn ruled_arguments_follow_rules() {
        let e = env();
        let args = vec![
            Argument::Ruled {
                rules: allow(),
                value: ArgumentValue::Many(vec![
                    "--width".to_string(),
                    "${resolution_width}".to_string(),
                ]),
            },
            Argument::Ruled {
                rules: deny(),
                value: ArgumentValue::Single("--skipped".to_string()),
            },
        ];
        let out = call(&e, Some(&args), None).unwrap();
        assert_eq!(out, vec!["--width", "1280"]);
    }

    #[test]
    fn ruled_feature_argument_for_custom_resolution_is_included() {
        let e = env();
        let args = vec![Argument::Ruled {
            rules: vec![Rule {
                action: RuleAction::Allow,
                os: None,
                features: Some(FeatureRule {
                    is_demo_user: None,
                    has_custom_resolution: Some(true),
                    has_quick_plays_support: None,
                    is_quick_play_singleplayer: None,
                    is_quick_play_multiplayer: None,
                    is_quick_play_realms: None,
                }),
            }],
            value: ArgumentValue::Many(vec![
                "--width".to_string(),
                "${resolution_width}".to_string(),
            ]),
        }];
        assert_eq!(
            call(&e, Some(&args), None).unwrap(),
            vec!["--width", "1280"]
        );
    }

    #[test]
    fn ruled_single_with_space_is_passed_verbatim_not_split() {
        let e = env();
        let args = vec![Argument::Ruled {
            rules: allow(),
            value: ArgumentValue::Single("--width ${resolution_width}".to_string()),
        }];
        assert_eq!(
            call(&e, Some(&args), None).unwrap(),
            vec!["--width 1280"]
        );
    }

    #[test]
    fn modern_arguments_win_over_legacy() {
        let e = env();
        let out = call(&e, Some(&[normal("--a")]), Some("--legacy")).unwrap();
        assert_eq!(out, vec!["--a"]);
    }

    #[test]
    fn empty_modern_arguments_win_over_legacy() {
        let e = env();
        assert!(call(&e, Some(&[]), Some("--legacy")).unwrap().is_empty());
    }

    #[test]
    fn modern_empty_list_does_not_validate_directories() {
        let e = Env {
            _tmp: tempfile::tempdir().unwrap(),
            game: "/definitely/missing/game".into(),
            assets: "/definitely/missing/assets".into(),
        };
        assert!(call(&e, Some(&[]), None).unwrap().is_empty());
    }

    #[test]
    fn legacy_arguments_are_split_on_spaces_and_replaced() {
        let e = env();
        let legacy = "--username ${auth_player_name} --session ${auth_session} --version ${version_name} --gameDir ${game_directory} --assetsDir ${game_assets} --uuid ${auth_uuid}";
        let out = call(&e, None, Some(legacy)).unwrap();
        assert_eq!(
            out,
            vec![
                "--username".to_string(),
                "Steve_1".to_string(),
                "--session".to_string(),
                "tok-123".to_string(),
                "--version".to_string(),
                "1.20.1".to_string(),
                "--gameDir".to_string(),
                canon(&e.game),
                "--assetsDir".to_string(),
                canon(&e.assets),
                "--uuid".to_string(),
                UUID_STR.to_string(),
            ]
        );
    }

    #[test]
    fn legacy_consecutive_spaces_give_empty_arguments() {
        let e = env();
        assert_eq!(call(&e, None, Some("a  b")).unwrap(), vec!["a", "", "b"]);
    }

    #[test]
    fn legacy_empty_string_gives_one_empty_argument() {
        let e = env();
        assert_eq!(call(&e, None, Some("")).unwrap(), vec![""]);
    }

    #[test]
    fn missing_game_directory_fails_with_game_entity() {
        let mut e = env();
        e.game = e.game.join("missing");
        for (args, legacy) in [(Some(vec![normal("x")]), None), (None, Some("x"))] {
            let err = call(&e, args.as_deref(), legacy).unwrap_err();
            assert!(matches!(
                err,
                MinecraftDomainError::PathNotFound { ref entity_type, .. } if entity_type == "game"
            ));
        }
    }

    #[test]
    fn missing_assets_directory_fails_with_assets_entity() {
        let mut e = env();
        e.assets = e.assets.join("missing");
        let err = call(&e, Some(&[normal("x")]), None).unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::PathNotFound { ref entity_type, .. } if entity_type == "assets"
        ));
    }

    #[test]
    fn version_type_variants_are_rendered_in_snake_case() {
        let e = env();
        for (vt, text) in [
            (VersionType::Release, "release"),
            (VersionType::Snapshot, "snapshot"),
            (VersionType::OldAlpha, "old_alpha"),
            (VersionType::OldBeta, "old_beta"),
        ] {
            let out = get_minecraft_arguments(
                Some(&[normal("${version_type}")]),
                None,
                &credential(),
                "v",
                "i",
                &e.game,
                &e.assets,
                &vt,
                WindowSize::default(),
                "x64",
            )
            .unwrap();
            assert_eq!(out, vec![text]);
        }
    }

    #[test]
    fn default_resolution_is_rendered() {
        let e = env();
        let out = get_minecraft_arguments(
            Some(&[normal("${resolution_width}x${resolution_height}")]),
            None,
            &credential(),
            "v",
            "i",
            &e.game,
            &e.assets,
            &VersionType::Release,
            WindowSize::default(),
            "x64",
        )
        .unwrap();
        assert_eq!(out, vec!["960x540"]);
    }
}
