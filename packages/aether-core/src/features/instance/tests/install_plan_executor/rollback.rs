//! Rolling back a failed update.

use std::collections::HashMap;

use crate::features::{
    instance::{InstallPlan, InstanceError, InstanceId, PackFile},
    minecraft::ModLoader,
};

use super::common::*;

/// The existing instance with two files of the pack: `mods/a.jar` (replaced by the plans below)
/// and `mods/old.jar` (dropped by them), and a file of the user's at `config/user.toml`.
async fn installed_pack(fixture: &Fixture) -> HashMap<String, PackFile> {
    let pack = pack_of(&plugin("packwiz"));
    fixture
        .put_installed("mods/a.jar", "a 1", Some(pack.clone()))
        .await;
    fixture
        .put_installed("mods/old.jar", "old", Some(pack))
        .await;
    fixture.put("config/user.toml", "user");
    fixture.records(INSTANCE_ID).await
}

/// The instance holds exactly what [`installed_pack`] put there, recorded as it was.
async fn assert_as_installed(fixture: &Fixture, records_before: &HashMap<String, PackFile>) {
    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["config/user.toml", "mods/a.jar", "mods/old.jar"]
    );
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "mods/a.jar").as_deref(),
        Some("a 1")
    );
    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "config/user.toml")
            .as_deref(),
        Some("user")
    );
    // `PackFile` has no `PartialEq`; these records have no update info, whose map would print in
    // any order.
    let printed = |records: &HashMap<String, PackFile>| {
        records
            .iter()
            .map(|(path, record)| (path.clone(), format!("{record:?}")))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    assert_eq!(
        printed(&fixture.records(INSTANCE_ID).await),
        printed(records_before)
    );
    assert!(fixture.staging_is_clean());
}

/// A planned file that cannot be moved in after others were: those moved in are taken out
/// again, the file they replaced is back, and nothing is recorded or removed.
#[tokio::test]
async fn should_roll_back_an_update_when_installing_a_file_fails() {
    let fixture = Fixture::new().await;
    let records_before = installed_pack(&fixture).await;
    // A file where the plan needs a directory.
    fixture.put("resourcepacks", "not a directory");

    let result = fixture
        .execute(
            existing(vec![
                hashed("https://cdn.example/a-2.jar", "mods/a.jar"),
                hashed("https://cdn.example/b.jar", "mods/b.jar"),
                hashed("https://cdn.example/pack.zip", "resourcepacks/pack.zip"),
            ]),
            &plugin("packwiz"),
        )
        .await;

    assert!(result.is_err());
    std::fs::remove_file(
        fixture
            .location_info
            .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .join("resourcepacks"),
    )
    .unwrap();
    assert_as_installed(&fixture, &records_before).await;
}

/// A dropped file that cannot be removed (a directory at its path, on any OS) keeps its record,
/// and the update is rolled back: the other dropped files come back with their records, and the
/// new files and records are gone.
#[tokio::test]
async fn should_roll_back_an_update_when_removing_a_dropped_file_fails() {
    let fixture = Fixture::new().await;
    let held = fixture
        .location_info
        .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
        .join("mods/held.jar");
    std::fs::create_dir_all(held.join("inner")).unwrap();
    fixture
        .record(
            "mods/held.jar",
            PackFile {
                installed_by: Some(pack_of(&plugin("packwiz"))),
                ..PackFile::from_hash("held.jar".into(), sha1("held"))
            },
        )
        .await;
    let records_before = installed_pack(&fixture).await;

    let err = fixture
        .execute(
            existing(vec![hashed("https://cdn.example/a-2.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(matches!(err, InstanceError::Storage(_)), "{err:?}");
    assert!(held.join("inner").is_dir());
    std::fs::remove_dir_all(&held).unwrap();
    assert_as_installed(&fixture, &records_before).await;
}

/// Overrides are applied last: one that cannot be written undoes the files installed, removed
/// and written over before it.
#[tokio::test]
async fn should_roll_back_an_update_when_applying_an_override_fails() {
    let fixture = Fixture::new().await;
    let records_before = installed_pack(&fixture).await;
    fixture.put("blocked", "not a directory");
    let archive = zip_archive(
        &fixture,
        &[
            ("overrides/config/user.toml", "pack"),
            ("overrides/config/new.toml", "new"),
            ("overrides/blocked/a.txt", "a"),
        ],
    );

    let result = fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![hashed("https://cdn.example/a-2.jar", "mods/a.jar")])
            },
            &plugin("packwiz"),
        )
        .await;

    assert!(result.is_err());
    std::fs::remove_file(
        fixture
            .location_info
            .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .join("blocked"),
    )
    .unwrap();
    assert_as_installed(&fixture, &records_before).await;
}

/// A disabled file that a failed update replaced comes back disabled, under its own name.
#[tokio::test]
async fn should_restore_a_disabled_file_when_an_update_fails() {
    let fixture = Fixture::new().await;
    fixture
        .put_installed("mods/a.jar", "a 1", Some(pack_of(&plugin("packwiz"))))
        .await;
    let dir = fixture
        .location_info
        .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap());
    std::fs::rename(dir.join("mods/a.jar"), dir.join("mods/a.jar.disabled")).unwrap();
    fixture.put("resourcepacks", "not a directory");

    fixture
        .execute(
            existing(vec![
                hashed("https://cdn.example/a-2.jar", "mods/a.jar"),
                hashed("https://cdn.example/pack.zip", "resourcepacks/pack.zip"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/a.jar.disabled", "resourcepacks"]
    );
    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "mods/a.jar.disabled")
            .as_deref(),
        Some("a 1")
    );
}

/// A pack update that moved the instance to another game and then failed moves it back: the
/// old game is still installed.
#[tokio::test]
async fn should_move_the_instance_back_to_its_game_when_an_update_fails() {
    let mut fixture = Fixture::new().await;
    fixture.pack_info = Some(installed_from(&plugin("packwiz"), PACK_ID));
    fixture.put("mods", "not a directory");

    fixture
        .execute(
            existing_on(
                &neoforge_1_21_4(),
                vec![hashed("https://cdn.example/a.jar", "mods/a.jar")],
            ),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert_eq!(fixture.change_game.changes().len(), 1);
    let upserted = fixture.upserted.lock().unwrap().clone();
    let [restored] = &upserted[..] else {
        panic!("one upsert expected: {upserted:?}")
    };
    assert_eq!(restored.game_version, "1.21.1");
    assert_eq!(restored.loader, ModLoader::Fabric);
}

/// A file that cannot be put back keeps its backup: the copy in the temp directory may be the
/// only one left of what the update replaced.
#[tokio::test]
async fn should_keep_the_backups_when_putting_a_file_back_fails() {
    let mut fixture = Fixture::new().await;
    fixture.fail_restore = true;
    installed_pack(&fixture).await;
    fixture.put("resourcepacks", "not a directory");

    let result = fixture
        .execute(
            existing(vec![
                hashed("https://cdn.example/a-2.jar", "mods/a.jar"),
                hashed("https://cdn.example/pack.zip", "resourcepacks/pack.zip"),
            ]),
            &plugin("packwiz"),
        )
        .await;

    assert!(result.is_err());
    let [backups] = &fixture.backup_dirs()[..] else {
        panic!("one backup directory expected: {:?}", fixture.backup_dirs())
    };
    let kept: Vec<String> = std::fs::read_dir(backups)
        .unwrap()
        .map(|entry| std::fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect();
    assert!(kept.contains(&"a 1".to_owned()), "{kept:?}");
    // Only the backups stay; the staging of the run is gone.
    assert_eq!(
        std::fs::read_dir(fixture.location_info.temp_dir())
            .unwrap()
            .count(),
        1
    );
}

/// A run dropped after it moved files aside cannot roll back: its backups stay where they are.
#[tokio::test]
async fn should_keep_the_backups_when_the_run_is_dropped_after_files_were_moved_aside() {
    let fixture = Fixture::new().await;
    *fixture.progress.stall_on.lock().unwrap() = Some("Installing".to_owned());
    installed_pack(&fixture).await;
    let packwiz = plugin("packwiz");
    let run = fixture.execute(
        existing(vec![hashed("https://cdn.example/a-2.jar", "mods/a.jar")]),
        &packwiz,
    );

    tokio::select! {
        result = run => panic!("the run cannot end: {result:?}"),
        () = fixture.progress.stalled.notified() => {}
    }

    let [backups] = &fixture.backup_dirs()[..] else {
        panic!("one backup directory expected: {:?}", fixture.backup_dirs())
    };
    assert_eq!(std::fs::read_to_string(backups.join("0")).unwrap(), "a 1");
    // The staging directory of the run is gone.
    assert_eq!(
        std::fs::read_dir(fixture.location_info.temp_dir())
            .unwrap()
            .count(),
        1
    );
}

/// The backups of a run that went through are gone with its staging directory.
#[tokio::test]
async fn should_leave_no_backups_when_an_update_succeeds() {
    let fixture = Fixture::new().await;
    installed_pack(&fixture).await;

    fixture
        .execute(
            existing(vec![hashed("https://cdn.example/a-2.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["config/user.toml", "mods/a.jar"]
    );
    assert!(fixture.staging_is_clean());
}

/// A failed download leaves the instance as it was: nothing removed, nothing recorded.
#[tokio::test]
async fn should_remove_nothing_when_a_download_fails() {
    let fixture = Fixture::new().await;
    let pack = pack_of(&plugin("packwiz"));
    fixture
        .put_installed("mods/a-1.jar", "a 1", Some(pack))
        .await;

    fixture
        .execute(
            existing(vec![
                hashed("https://cdn.example/a-2.jar", "mods/a-2.jar"),
                hashed("https://cdn.example/fail.jar", "mods/b.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a-1.jar"]);
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records.len(), 1);
    assert!(records.contains_key("mods/a-1.jar"));
}

/// Running the same plan twice changes nothing the second time.
#[tokio::test]
async fn should_be_a_no_op_when_the_plan_runs_a_second_time() {
    let fixture = Fixture::new().await;
    let plan = || {
        existing(vec![
            hashed("https://cdn.example/a.jar", "mods/a.jar"),
            hashed("https://cdn.example/b.jar", "mods/b.jar"),
        ])
    };

    fixture.execute(plan(), &plugin("packwiz")).await.unwrap();
    fixture.execute(plan(), &plugin("packwiz")).await.unwrap();

    assert_eq!(fixture.downloader.downloads().len(), 2);
    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/a.jar", "mods/b.jar"]
    );
}
