//! Changing the game of an instance with a pack update.

use crate::features::instance::{InstallPlan, InstanceError, InstanceGame};

use super::common::*;

/// An update of the instance's own pack that moves to another game: refs are resolved for the
/// new game, not the one the instance runs now, and the instance is moved to it.
#[tokio::test]
async fn should_resolve_refs_against_the_game_when_a_pack_update_moves_to_it() {
    let mut fixture = Fixture::new().await;
    fixture.pack_info = Some(installed_from(&plugin("packwiz"), PACK_ID));
    let modrinth = MockProvider::builtin_modrinth();
    fixture.register(&modrinth).await;

    fixture
        .execute(
            existing_on(
                &neoforge_1_21_4(),
                vec![provider_ref("modrinth", "AANobbMI", "mods/sodium.jar")],
            ),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        modrinth.calls(),
        [(
            "AANobbMI".into(),
            None,
            "1.21.4".into(),
            Some("neoforge".into())
        )]
    );
    assert_eq!(
        fixture.change_game.changes(),
        [(INSTANCE_ID.to_owned(), neoforge_1_21_4())]
    );
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "mods/sodium.jar"),
        Some(modrinth.url_for("AANobbMI"))
    );
}

/// Only the pack the instance was installed from may move it to another game; any other plan
/// naming a game is refused before a ref is resolved, an archive read or a file downloaded.
#[tokio::test]
async fn should_refuse_a_game_change_when_the_plan_is_of_another_content() {
    for pack_info in [
        None,
        Some(installed_from(&plugin("packwiz"), "another-pack")),
        Some(installed_from(&plugin("other"), PACK_ID)),
    ] {
        let mut fixture = Fixture::new().await;
        fixture.pack_info = pack_info.clone();
        let modrinth = MockProvider::builtin_modrinth();
        fixture.register(&modrinth).await;

        let plan = InstallPlan {
            // An archive that does not exist: reading it would fail with another error.
            overrides: archive_overrides(fixture.location_info.temp_dir().join("missing.zip")),
            ..existing_on(
                &neoforge_1_21_4(),
                vec![
                    provider_ref("modrinth", "AANobbMI", "mods/sodium.jar"),
                    direct("https://cdn.example/pack/a.jar", "mods/a.jar"),
                ],
            )
        };
        let err = fixture.execute(plan, &plugin("packwiz")).await.unwrap_err();

        assert!(
            matches!(
                &err,
                InstanceError::GameChangeNotAllowed { instance_id, provider_id, content_id }
                    if instance_id == INSTANCE_ID
                        && *provider_id == plugin("packwiz")
                        && content_id == PACK_ID
            ),
            "{pack_info:?}: {err:?}"
        );
        assert!(modrinth.calls().is_empty(), "{pack_info:?}");
        assert!(fixture.downloaded_urls().is_empty(), "{pack_info:?}");
        assert!(fixture.change_game.changes().is_empty(), "{pack_info:?}");
        assert!(
            fixture.instance_files(INSTANCE_ID).is_empty(),
            "{pack_info:?}"
        );
    }
}

#[tokio::test]
async fn should_keep_the_game_when_a_download_fails() {
    let mut fixture = Fixture::new().await;
    fixture.pack_info = Some(installed_from(&plugin("packwiz"), PACK_ID));

    fixture
        .execute(
            existing_on(
                &neoforge_1_21_4(),
                vec![
                    direct("https://cdn.example/pack/a.jar", "mods/a.jar"),
                    direct("https://cdn.example/pack/fail.jar", "mods/fail.jar"),
                ],
            ),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(fixture.change_game.changes().is_empty());
    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
    assert!(fixture.staging_is_clean());
}

/// The game is changed before any file is moved in: when installing the new game fails, the
/// instance keeps its old files as well.
#[tokio::test]
async fn should_install_no_file_when_moving_to_the_new_game_fails() {
    let mut fixture = Fixture::new().await;
    fixture.pack_info = Some(installed_from(&plugin("packwiz"), PACK_ID));
    let game = InstanceGame {
        game_version: "1.21.fail".into(),
        ..neoforge_1_21_4()
    };

    fixture
        .execute(
            existing_on(
                &game,
                vec![direct("https://cdn.example/pack/a.jar", "mods/a.jar")],
            ),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert_eq!(fixture.change_game.changes().len(), 1);
    assert_eq!(
        fixture.downloaded_urls(),
        ["https://cdn.example/pack/a.jar"]
    );
    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
    assert!(fixture.records(INSTANCE_ID).await.is_empty());
    assert!(fixture.staging_is_clean());
}

/// A plan without a game never moves the instance, whatever pack it was installed from.
#[tokio::test]
async fn should_keep_the_game_when_the_plan_names_none() {
    let mut fixture = Fixture::new().await;
    fixture.pack_info = Some(installed_from(&plugin("packwiz"), PACK_ID));

    fixture
        .execute(
            existing(vec![direct("https://cdn.example/pack/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert!(fixture.change_game.changes().is_empty());
}

/// `a.jar.disabled` is the disabled state of `a.jar`: both would end up as one file.
#[tokio::test]
async fn should_refuse_file_and_its_disabled_variant_when_planned_together() {
    let fixture = Fixture::new().await;

    let err = fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/a.jar", "mods/a.jar"),
                direct("https://cdn.example/b.jar", "mods/a.jar.disabled"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::ContentDuplication { .. }),
        "{err:?}"
    );
    assert!(fixture.downloader.downloads().is_empty());
}
