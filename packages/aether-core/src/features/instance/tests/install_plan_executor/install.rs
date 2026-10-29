//! Targets of a plan, its limits, and checking it as data.

use tempfile::TempDir;

use crate::features::{
    instance::{
        FileSide, InstallFileEntry, InstallPlan, InstallTarget, InstanceError, InstanceId,
        NewInstance, OptionalFile, PackInfo, ProviderId, remove_plan_leftovers,
    },
    minecraft::ModLoader,
    settings::LocationInfo,
};

use super::common::*;

/// A new instance is created through the create use case, with the spec as the plan gives it,
/// and its game is what refs are resolved against — before it exists.
#[tokio::test]
async fn should_create_a_new_instance_and_install_into_it_when_the_target_is_new() {
    let fixture = Fixture::new().await;
    let modrinth = MockProvider::builtin_modrinth();
    fixture.register(&modrinth).await;
    let spec = NewInstance {
        name: "Pack".into(),
        game_version: "1.20.1".into(),
        mod_loader: ModLoader::Vanilla,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: None,
    };

    let instance_id = fixture
        .execute(
            InstallPlan {
                target: InstallTarget::New(spec.clone()),
                files: vec![provider_ref("modrinth", "AANobbMI", "mods/sodium.jar")],
                overrides: Vec::new(),
            },
            &ProviderId::from_parts(ProviderId::BUILTIN_PLUGIN_ID, "modrinth").unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instance_id.as_str(), NEW_INSTANCE_ID);
    assert_eq!(*fixture.create_instance.created.lock().unwrap(), [spec]);
    assert_eq!(
        modrinth.calls(),
        [("AANobbMI".into(), None, "1.20.1".into(), None)]
    );
    assert_eq!(fixture.instance_files(NEW_INSTANCE_ID), ["mods/sodium.jar"]);
}

/// A new instance of a pack is created for it: not launchable until the pack is in.
#[tokio::test]
async fn should_create_the_instance_of_a_pack_when_the_plan_is_for_a_pack() {
    let fixture = Fixture::new().await;
    let spec = NewInstance {
        name: "Pack".into(),
        game_version: "1.20.1".into(),
        mod_loader: ModLoader::Vanilla,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: Some(PackInfo {
            provider_id: plugin("packwiz"),
            modpack_id: "pack".into(),
            version_id: "1".into(),
        }),
    };

    fixture
        .execute(
            InstallPlan {
                target: InstallTarget::New(spec.clone()),
                files: vec![direct("https://cdn.example/a.jar", "mods/a.jar")],
                overrides: Vec::new(),
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        *fixture.create_instance.created_for_pack.lock().unwrap(),
        [spec]
    );
    assert!(fixture.create_instance.created.lock().unwrap().is_empty());
}

/// A file where the plan needs a directory: moving any planned mod into the instance fails.
fn block_mods_dir(location_info: &LocationInfo, instance_id: &str) {
    let dir = location_info.instance_dir(&InstanceId::try_from(instance_id).unwrap());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("mods"), "not a directory").unwrap();
}

/// A new pack's instance that cannot be filled is removed, not left `PackInstalling` for good.
#[tokio::test]
async fn should_remove_a_new_instance_when_it_could_not_be_filled() {
    let fixture = Fixture::new().await;
    block_mods_dir(&fixture.location_info, NEW_INSTANCE_ID);

    let err = fixture
        .execute(
            InstallPlan {
                target: InstallTarget::New(NewInstance {
                    name: "Pack".into(),
                    game_version: "1.20.1".into(),
                    mod_loader: ModLoader::Vanilla,
                    loader_version: None,
                    icon_path: None,
                    skip_install_instance: Some(true),
                    pack_info: Some(installed_from(&plugin("packwiz"), PACK_ID)),
                }),
                files: vec![direct("https://cdn.example/a.jar", "mods/a.jar")],
                overrides: Vec::new(),
            },
            &plugin("packwiz"),
        )
        .await;

    assert!(err.is_err());
    assert_eq!(*fixture.remove.removed.lock().unwrap(), [NEW_INSTANCE_ID]);
}

/// An existing instance is the user's: a failed update never removes it.
#[tokio::test]
async fn should_keep_an_existing_instance_when_it_could_not_be_filled() {
    let fixture = Fixture::new().await;
    block_mods_dir(&fixture.location_info, INSTANCE_ID);

    let err = fixture
        .execute(
            existing(vec![direct("https://cdn.example/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await;

    assert!(err.is_err());
    assert!(fixture.remove.removed.lock().unwrap().is_empty());
}

#[tokio::test]
async fn should_not_create_the_instance_when_a_download_fails() {
    let fixture = Fixture::new().await;

    let err = fixture
        .execute(
            InstallPlan {
                target: InstallTarget::New(NewInstance {
                    name: "Pack".into(),
                    game_version: "1.20.1".into(),
                    mod_loader: ModLoader::Fabric,
                    loader_version: None,
                    icon_path: None,
                    skip_install_instance: Some(true),
                    pack_info: None,
                }),
                files: vec![
                    direct("https://cdn.example/a.jar", "mods/a.jar"),
                    direct("https://cdn.example/fail.jar", "mods/b.jar"),
                ],
                overrides: Vec::new(),
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::ContentDownloadError(_)),
        "{err:?}"
    );
    assert!(fixture.create_instance.created.lock().unwrap().is_empty());
}

/// One failed download and the instance stays as it was: the files that did download stay in
/// the staging directory.
///
/// Whether that directory is gone afterwards is not asserted here: the failure cancels the
/// other download, whose write may still land after the cleanup.
#[tokio::test]
async fn should_write_nothing_into_the_instance_when_a_download_fails() {
    let fixture = Fixture::new().await;

    let err = fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/a.jar", "mods/a.jar"),
                direct("https://cdn.example/fail.jar", "mods/b.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::ContentDownloadError(_)),
        "{err:?}"
    );
    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
}

/// A plan of more files than the limit is refused before anything of it is resolved.
#[tokio::test]
async fn should_refuse_a_plan_when_it_lists_more_files_than_the_limit() {
    let mut fixture = Fixture::new().await;
    fixture.plan_limits = Some((2, u64::MAX));
    let modrinth = MockProvider::builtin_modrinth();
    fixture.register(&modrinth).await;

    let err = fixture
        .execute(
            existing(vec![
                provider_ref("modrinth", "a", "mods/a.jar"),
                provider_ref("modrinth", "b", "mods/b.jar"),
                provider_ref("modrinth", "c", "mods/c.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::ContentProviderError { .. }),
        "{err:?}"
    );
    assert!(modrinth.calls().is_empty());
    assert!(fixture.downloaded_urls().is_empty());
}

/// Files declaring more bytes together than the plan may download are refused before any is.
#[tokio::test]
async fn should_refuse_a_plan_when_its_files_declare_more_bytes_than_the_limit() {
    let mut fixture = Fixture::new().await;
    fixture.plan_limits = Some((100, 150));
    let sized = |url: &str, path: &str| {
        let mut entry = direct(url, path);
        entry.size = Some(100);
        entry
    };

    let err = fixture
        .execute(
            existing(vec![
                sized("https://cdn.example/a.jar", "mods/a.jar"),
                sized("https://cdn.example/b.jar", "mods/b.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::FileTooLarge { limit: 150, .. }),
        "{err:?}"
    );
    assert!(fixture.downloaded_urls().is_empty());
    assert_eq!(fixture.instance_files(INSTANCE_ID), Vec::<String>::new());
}

/// Sizes the plan does not declare are counted as the bytes arrive: each file fits, both do
/// not, and nothing gets into the instance.
#[tokio::test]
async fn should_refuse_a_plan_when_its_downloads_bring_more_bytes_than_the_limit() {
    let first = "https://cdn.example/a.jar";
    let second = "https://cdn.example/chunks-b.jar";
    let mut fixture = Fixture::new().await;
    fixture.plan_limits = Some((100, (first.len() + second.len() - 1) as u64));

    let err = fixture
        .execute(
            existing(vec![
                direct(first, "mods/a.jar"),
                direct(second, "mods/b.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::FileTooLarge { .. }),
        "{err:?}"
    );
    assert_eq!(fixture.instance_files(INSTANCE_ID), Vec::<String>::new());
}

/// A run dropped halfway — its task cancelled — takes its staging directory with it.
#[tokio::test]
async fn should_remove_the_staging_when_the_run_is_dropped_halfway() {
    let fixture = Fixture::new().await;
    let packwiz = plugin("packwiz");
    let run = fixture.execute(
        // One file: a concurrent download's write could land while the guard removes staging.
        existing(vec![direct(
            "https://cdn.example/hang.jar",
            "mods/hang.jar",
        )]),
        &packwiz,
    );

    tokio::select! {
        result = run => panic!("the run cannot end: {result:?}"),
        () = fixture.downloader.hanging.notified() => {}
    }

    assert!(fixture.staging_is_clean());
    assert_eq!(fixture.instance_files(INSTANCE_ID), Vec::<String>::new());
}

/// At startup the directories of runs a crash interrupted go, but for backups that hold files:
/// they may be the only copy of what a run moved aside.
#[tokio::test]
async fn should_remove_the_leftovers_of_interrupted_runs_but_keep_the_backups_with_files() {
    let temp = TempDir::new().unwrap();
    let dir = |name: &str| temp.path().join(name);
    std::fs::create_dir_all(dir("install-plan-1/overrides")).unwrap();
    std::fs::write(dir("install-plan-1/0"), "staged").unwrap();
    std::fs::create_dir(dir("install-plan-1-backup")).unwrap();
    std::fs::create_dir(dir("install-plan-2-backup")).unwrap();
    std::fs::write(dir("install-plan-2-backup/0"), "the user's file").unwrap();
    std::fs::create_dir(dir("other")).unwrap();
    std::fs::write(dir("install-plan-file"), "not a run").unwrap();

    remove_plan_leftovers(temp.path()).await;

    let mut left: Vec<String> = std::fs::read_dir(temp.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(
        left,
        ["install-plan-2-backup", "install-plan-file", "other"]
    );
    assert_eq!(
        std::fs::read_to_string(dir("install-plan-2-backup/0")).unwrap(),
        "the user's file"
    );
}

#[tokio::test]
async fn should_refuse_a_plan_when_the_instance_is_unknown() {
    let fixture = Fixture::new().await;

    let err = fixture
        .execute(
            InstallPlan {
                target: InstallTarget::existing(InstanceId::try_from("missing").unwrap()),
                files: vec![direct("https://cdn.example/a.jar", "mods/a.jar")],
                overrides: Vec::new(),
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(matches!(err, InstanceError::NotFound { .. }), "{err:?}");
    assert!(fixture.downloader.downloads().is_empty());
}

// ─── Checking the plan ─────────────────────────────────────────

/// A client instance skips server-only files, and an optional file is taken only when it is on
/// by default.
#[tokio::test]
async fn should_install_the_default_selection_of_files_when_no_choice_is_made() {
    let fixture = Fixture::new().await;
    let optional = |default_enabled| {
        Some(OptionalFile {
            default_enabled,
            description: None,
        })
    };

    fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/both.jar", "mods/both.jar"),
                InstallFileEntry {
                    side: FileSide::Client,
                    ..direct("https://cdn.example/client.jar", "mods/client.jar")
                },
                InstallFileEntry {
                    side: FileSide::Server,
                    ..direct("https://cdn.example/server.jar", "mods/server.jar")
                },
                InstallFileEntry {
                    optional: optional(true),
                    ..direct("https://cdn.example/on.jar", "mods/on.jar")
                },
                InstallFileEntry {
                    optional: optional(false),
                    ..direct("https://cdn.example/off.jar", "mods/off.jar")
                },
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/both.jar", "mods/client.jar", "mods/on.jar"]
    );
}

#[tokio::test]
async fn should_refuse_a_path_before_downloading_anything_when_it_leads_out_of_the_instance() {
    for path in [
        "../escape.jar",
        "/etc/passwd",
        "mods/../../escape.jar",
        "C:/x.jar",
    ] {
        let fixture = Fixture::new().await;

        let err = fixture
            .execute(
                existing(vec![
                    direct("https://cdn.example/a.jar", "mods/a.jar"),
                    direct("https://cdn.example/b.jar", path),
                ]),
                &plugin("packwiz"),
            )
            .await
            .unwrap_err();

        assert!(
            matches!(err, InstanceError::InvalidRelativePath { .. }),
            "{path}: {err:?}"
        );
        assert!(fixture.downloader.downloads().is_empty(), "{path}");
    }
}

/// `.metadata` is the launcher's: no plan writes there, the built-in provider's included.
#[tokio::test]
async fn should_refuse_a_path_when_it_leads_into_the_instance_metadata() {
    for path in [
        ".metadata/instance.json",
        ".METADATA/instance.json",
        ".metadata/plugins/packwiz/packwiz.toml",
    ] {
        let fixture = Fixture::new().await;

        let err = fixture
            .execute(
                existing(vec![direct("https://cdn.example/a.json", path)]),
                &ProviderId::from_parts(ProviderId::BUILTIN_PLUGIN_ID, "modrinth").unwrap(),
            )
            .await
            .unwrap_err();

        assert!(
            matches!(err, InstanceError::InvalidRelativePath { .. }),
            "{path}: {err:?}"
        );
        assert!(fixture.downloader.downloads().is_empty(), "{path}");
    }
}

#[tokio::test]
async fn should_refuse_two_files_when_they_share_one_path() {
    let fixture = Fixture::new().await;

    let err = fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/a.jar", "mods/a.jar"),
                direct("https://cdn.example/b.jar", "mods\\a.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::ContentDuplication { content_path } if content_path == "mods/a.jar"),
        "{err:?}"
    );
    assert!(fixture.downloader.downloads().is_empty());
}

/// On a case-insensitive file system these are one file: the second would replace the first.
#[tokio::test]
async fn should_refuse_two_files_when_their_paths_differ_only_in_case() {
    let fixture = Fixture::new().await;

    let err = fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/a.jar", "mods/Foo.jar"),
                direct("https://cdn.example/b.jar", "mods/foo.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::ContentDuplication { content_path } if content_path == "mods/foo.jar"),
        "{err:?}"
    );
    assert!(fixture.downloader.downloads().is_empty());
}
