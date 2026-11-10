//! The game a plan runs is downloaded before the point of no return, and only when it is not
//! the one the target runs already.
use crate::features::{
    instance::{InstallTarget, InstanceGame},
    minecraft::{GameVersion, ModLoader},
};

use super::{cancel::new_instance_with_a_game_plan, common::*};

#[tokio::test]
async fn should_download_the_game_when_a_new_instance_is_installed() {
    let fixture = Fixture::new().await;

    fixture
        .execute(new_instance_with_a_game_plan(), &plugin("packwiz"))
        .await
        .unwrap();

    assert_eq!(
        *fixture.game_download.downloads.lock().unwrap(),
        [GameVersion::from_static("1.20.1")]
    );
    assert_eq!(fixture.create_instance.created.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn should_download_no_game_when_the_game_is_not_installed_with_the_instance() {
    let fixture = Fixture::new().await;
    let mut plan = new_instance_with_a_game_plan();
    let InstallTarget::New(spec) = &mut plan.target else {
        unreachable!()
    };
    spec.skip_install_instance = Some(true);

    fixture.execute(plan, &plugin("packwiz")).await.unwrap();

    assert!(fixture.game_download.downloads.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_download_no_game_when_the_instance_keeps_its_game() {
    let fixture = Fixture::new().await;

    fixture
        .execute(
            existing(vec![direct("https://cdn.example/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert!(fixture.game_download.downloads.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_download_no_game_when_the_plan_names_the_game_the_instance_runs() {
    let mut fixture = Fixture::new().await;
    fixture.pack_info = Some(installed_from(&plugin("packwiz"), PACK_ID));
    // The instance of the fixture runs Fabric on 1.21.1.
    let same = InstanceGame {
        game_version: GameVersion::from_static("1.21.1"),
        mod_loader: ModLoader::Fabric,
        loader_version: None,
    };

    fixture
        .execute(existing_on(&same, Vec::new()), &plugin("packwiz"))
        .await
        .unwrap();

    assert!(fixture.game_download.downloads.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_download_the_new_game_when_a_pack_update_moves_the_instance() {
    let mut fixture = Fixture::new().await;
    fixture.pack_info = Some(installed_from(&plugin("packwiz"), PACK_ID));

    fixture
        .execute(
            existing_on(&neoforge_1_21_4(), Vec::new()),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        *fixture.game_download.downloads.lock().unwrap(),
        [GameVersion::from_static("1.21.4")]
    );
}
