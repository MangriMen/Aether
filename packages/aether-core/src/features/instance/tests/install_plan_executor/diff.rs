//! Diffing a plan against what the instance already has.

use crate::{
    features::{
        instance::{
            ContentSource, ContentType, InstallFileEntry, InstallPlan, InstallTarget, NewInstance,
            PackFile, PlanOrigin,
        },
        minecraft::ModLoader,
    },
    shared::hash::FileHash,
};

use super::common::*;

#[tokio::test]
async fn should_record_installed_files_under_the_plan_origin_when_the_plan_succeeds() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    let url = "https://cdn.example/a.jar";

    fixture
        .execute(
            existing(vec![
                hashed(url, "mods/a.jar"),
                provider_ref("modrinth", "AANobbMI", "mods/sodium.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records.len(), 2);

    let a = &records["mods/a.jar"];
    assert_eq!(a.installed_by, Some(pack_of(&plugin("packwiz"))));
    assert_eq!(a.file_name, "a.jar");
    assert_eq!(a.hash, sha1(url));
    assert!(a.update.is_none());

    // A ref's file is recorded with what it resolved to, under the provider that resolved it.
    let sodium = &records["mods/sodium.jar"];
    assert_eq!(sodium.installed_by, Some(pack_of(&plugin("packwiz"))));
    assert_eq!(sodium.update_provider_id, Some(modrinth_id()));
    let info = &sodium.update.as_ref().unwrap()[&modrinth_id()];
    assert_eq!(
        (info.content_id.as_str(), info.version.as_str()),
        ("AANobbMI", "newest")
    );
}

#[tokio::test]
async fn should_record_the_files_of_a_new_instance_when_it_is_created() {
    let fixture = Fixture::new().await;

    fixture
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
                files: vec![direct("https://cdn.example/a.jar", "mods/a.jar")],
                overrides: Vec::new(),
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    let records = fixture.records(NEW_INSTANCE_ID).await;
    assert_eq!(
        records["mods/a.jar"].installed_by,
        Some(pack_of(&plugin("packwiz")))
    );
    // Read from the file: the plan had no hash to take it from.
    assert_eq!(
        records["mods/a.jar"].hash,
        sha1("https://cdn.example/a.jar")
    );
}

/// New file: what the instance does not have is downloaded.
/// Same hash: what it already has is not downloaded again, and stays recorded as the pack's.
#[tokio::test]
async fn should_download_only_what_the_instance_does_not_have_when_files_are_present() {
    let fixture = Fixture::new().await;
    let pack = pack_of(&plugin("packwiz"));
    let kept = "https://cdn.example/kept.jar";
    fixture
        .put_installed("mods/kept.jar", kept, Some(pack.clone()))
        .await;

    fixture
        .execute(
            existing(vec![
                hashed(kept, "mods/kept.jar"),
                hashed("https://cdn.example/new.jar", "mods/new.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.downloaded_urls(), ["https://cdn.example/new.jar"]);
    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/kept.jar", "mods/new.jar"]
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records["mods/kept.jar"].installed_by, Some(pack.clone()));
    assert_eq!(records["mods/new.jar"].installed_by, Some(pack));
}

/// The file on disk is what is checked, with the plan's own algorithm — not the recorded hash,
/// which may be stale.
#[tokio::test]
async fn should_check_the_file_on_disk_with_the_plan_algorithm_when_it_exists() {
    let fixture = Fixture::new().await;
    let url = "https://cdn.example/a.jar";
    fixture.put("mods/a.jar", url);
    fixture
        .record(
            "mods/a.jar",
            PackFile::from_hash("a.jar".into(), "stale".into()),
        )
        .await;
    let sha512 = crate::shared::hash::infra::hash_hex_async(
        crate::shared::hash::HashAlgorithm::Sha512,
        url.as_bytes().to_vec(),
    )
    .await
    .unwrap();

    fixture
        .execute(
            existing(vec![InstallFileEntry::new(
                ContentSource::direct(
                    url.into(),
                    Some(FileHash::parse("sha512", &sha512).unwrap()),
                ),
                "mods/a.jar",
                ContentType::Mod,
            )]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert!(fixture.downloaded_urls().is_empty());
    // The record now holds the file's sha1, read from it.
    assert_eq!(
        fixture.records(INSTANCE_ID).await["mods/a.jar"].hash,
        sha1(url)
    );
}

/// Changed hash: the file at the path is replaced.
#[tokio::test]
async fn should_replace_a_file_when_its_hash_changed() {
    let fixture = Fixture::new().await;
    let pack = pack_of(&plugin("packwiz"));
    fixture
        .put_installed("mods/a.jar", "old version", Some(pack))
        .await;
    let url = "https://cdn.example/a-2.jar";

    fixture
        .execute(
            existing(vec![hashed(url, "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.downloaded_urls(), [url]);
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "mods/a.jar").as_deref(),
        Some(url)
    );
    assert_eq!(
        fixture.records(INSTANCE_ID).await["mods/a.jar"].hash,
        sha1(url)
    );
}

/// Without a hash the file on disk cannot be told from the planned one: it is downloaded again.
#[tokio::test]
async fn should_download_again_a_file_when_the_plan_has_no_hash_for_it() {
    let fixture = Fixture::new().await;
    let url = "https://cdn.example/a.jar";
    fixture
        .put_installed("mods/a.jar", url, Some(pack_of(&plugin("packwiz"))))
        .await;

    fixture
        .execute(
            existing(vec![direct(url, "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.downloaded_urls(), [url]);
}

/// Dropped file: what the pack installed and no longer has is removed, from disk and record.
#[tokio::test]
async fn should_remove_a_file_when_the_pack_no_longer_has_it() {
    let fixture = Fixture::new().await;
    fixture
        .put_installed("mods/a-1.jar", "a 1", Some(pack_of(&plugin("packwiz"))))
        .await;

    fixture
        .execute(
            existing(vec![hashed("https://cdn.example/a-2.jar", "mods/a-2.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a-2.jar"]);
    let records = fixture.records(INSTANCE_ID).await;
    assert!(!records.contains_key("mods/a-1.jar"));
    assert!(records.contains_key("mods/a-2.jar"));
}

/// A record of the pack at a path no file of the instance can have is the storage's word only:
/// it is left alone, and the update goes through.
#[tokio::test]
async fn should_keep_a_dropped_record_when_its_path_is_out_of_the_content_area() {
    let fixture = Fixture::new().await;
    let pack = pack_of(&plugin("packwiz"));
    fixture
        .record(
            ".metadata/stale.jar",
            PackFile {
                installed_by: Some(pack.clone()),
                ..PackFile::from_hash("stale.jar".into(), sha1("stale"))
            },
        )
        .await;
    fixture
        .put_installed("mods/a-1.jar", "a 1", Some(pack))
        .await;

    fixture
        .execute(
            existing(vec![hashed("https://cdn.example/a-2.jar", "mods/a-2.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a-2.jar"]);
    let records = fixture.records(INSTANCE_ID).await;
    assert!(records.contains_key(".metadata/stale.jar"));
    assert!(!records.contains_key("mods/a-1.jar"));
}

/// Foreign files: added by hand (recorded by `list_content` without an owner, or not recorded
/// at all), or installed by another plan — another pack, a mod installed on its own, the same
/// content id from another provider. A pack update touches none of them.
#[tokio::test]
async fn should_leave_files_alone_when_the_pack_did_not_install_them() {
    let fixture = Fixture::new().await;
    let pack = pack_of(&plugin("packwiz"));
    fixture
        .put_installed("mods/by-hand.jar", "by hand", None)
        .await;
    fixture.put("mods/unlisted.jar", "unlisted");
    fixture.put("config/by-hand.json", "{}");
    fixture
        .put_installed(
            "mods/other-pack.jar",
            "other pack",
            Some(PlanOrigin {
                provider_id: plugin("packwiz"),
                content_id: "other-pack".into(),
            }),
        )
        .await;
    fixture
        .put_installed(
            "mods/other-provider.jar",
            "other provider",
            Some(pack_of(&plugin("other"))),
        )
        .await;
    fixture
        .put_installed(
            "mods/sodium.jar",
            "sodium",
            Some(PlanOrigin {
                provider_id: modrinth_id(),
                content_id: "AANobbMI".into(),
            }),
        )
        .await;

    fixture
        .execute(
            existing(vec![hashed("https://cdn.example/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        [
            "config/by-hand.json",
            "mods/a.jar",
            "mods/by-hand.jar",
            "mods/other-pack.jar",
            "mods/other-provider.jar",
            "mods/sodium.jar",
            "mods/unlisted.jar",
        ]
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records.len(), 5);
    assert_eq!(records["mods/by-hand.jar"].installed_by, None);
    assert_eq!(records["mods/a.jar"].installed_by, Some(pack));
}

/// A mod installed on its own is a plan of its own content: installing it into a pack's
/// instance removes none of the pack's files.
#[tokio::test]
async fn should_not_remove_pack_files_when_the_plan_is_for_a_single_mod() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    fixture
        .put_installed("mods/a.jar", "a", Some(pack_of(&plugin("packwiz"))))
        .await;

    fixture
        .execute_as(
            existing(vec![provider_ref(
                "modrinth",
                "AANobbMI",
                "mods/sodium.jar",
            )]),
            PlanOrigin {
                provider_id: modrinth_id(),
                content_id: "AANobbMI".into(),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/a.jar", "mods/sodium.jar"]
    );
}

/// A file the plan wants, already there by hand byte for byte, is the plan's from now on.
#[tokio::test]
async fn should_take_over_an_identical_file_when_it_sits_at_a_planned_path() {
    let fixture = Fixture::new().await;
    let url = "https://cdn.example/a.jar";
    fixture.put_installed("mods/a.jar", url, None).await;

    fixture
        .execute(
            existing(vec![hashed(url, "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert!(fixture.downloaded_urls().is_empty());
    assert_eq!(
        fixture.records(INSTANCE_ID).await["mods/a.jar"].installed_by,
        Some(pack_of(&plugin("packwiz")))
    );
}

/// A mod recorded without an owner, but whose update info names it: a plan for that very
/// content replaces it. A pack plan does not.
#[tokio::test]
async fn should_replace_an_older_install_when_it_is_of_the_same_content() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    let legacy = |path: &str| PackFile {
        update_provider_id: Some(modrinth_id()),
        update: Some(modrinth_update("AANobbMI", "v1")),
        ..PackFile::from_hash(file_name(path), sha1("old"))
    };
    fixture.put("mods/sodium-1.jar", "old");
    fixture
        .record("mods/sodium-1.jar", legacy("mods/sodium-1.jar"))
        .await;

    // The pack's plan leaves it alone.
    fixture
        .execute(
            existing(vec![hashed("https://cdn.example/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();
    assert!(
        fixture
            .instance_files(INSTANCE_ID)
            .contains(&"mods/sodium-1.jar".to_owned())
    );

    // The mod's own plan replaces it.
    fixture
        .execute_as(
            existing(vec![provider_ref(
                "modrinth",
                "AANobbMI",
                "mods/sodium-2.jar",
            )]),
            PlanOrigin {
                provider_id: modrinth_id(),
                content_id: "AANobbMI".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/a.jar", "mods/sodium-2.jar"]
    );
}

/// Disabled file, unchanged: `<path>.disabled` is the planned file — not downloaded, not
/// duplicated next to it.
#[tokio::test]
async fn should_keep_a_disabled_file_when_it_did_not_change() {
    let fixture = Fixture::new().await;
    let url = "https://cdn.example/a.jar";
    fixture.put("mods/a.jar.disabled", url);
    fixture
        .record(
            "mods/a.jar",
            PackFile {
                installed_by: Some(pack_of(&plugin("packwiz"))),
                ..PackFile::from_hash("a.jar".into(), sha1(url))
            },
        )
        .await;

    fixture
        .execute(
            existing(vec![hashed(url, "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert!(fixture.downloaded_urls().is_empty());
    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a.jar.disabled"]);
    assert!(
        fixture
            .records(INSTANCE_ID)
            .await
            .contains_key("mods/a.jar")
    );
}

/// Disabled file, changed: replaced in place, still disabled.
#[tokio::test]
async fn should_replace_a_disabled_file_and_keep_it_disabled_when_it_changed() {
    let fixture = Fixture::new().await;
    fixture.put("mods/a.jar.disabled", "old version");
    fixture
        .record(
            "mods/a.jar",
            PackFile {
                installed_by: Some(pack_of(&plugin("packwiz"))),
                ..PackFile::from_hash("a.jar".into(), sha1("old version"))
            },
        )
        .await;
    let url = "https://cdn.example/a-2.jar";

    fixture
        .execute(
            existing(vec![hashed(url, "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a.jar.disabled"]);
    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "mods/a.jar.disabled")
            .as_deref(),
        Some(url)
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records.len(), 1);
    assert_eq!(records["mods/a.jar"].hash, sha1(url));
}

/// Disabled file, new version under a new name: the ref names the same content as the dropped
/// disabled file, so the new version starts disabled too.
#[tokio::test]
async fn should_carry_the_disabled_state_to_a_new_version_when_the_content_is_the_same() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    fixture.put("mods/sodium-1.jar.disabled", "sodium 1");
    fixture
        .record(
            "mods/sodium-1.jar",
            PackFile {
                installed_by: Some(pack_of(&plugin("packwiz"))),
                update_provider_id: Some(modrinth_id()),
                update: Some(modrinth_update("AANobbMI", "v1")),
                ..PackFile::from_hash("sodium-1.jar".into(), sha1("sodium 1"))
            },
        )
        .await;

    fixture
        .execute(
            existing(vec![
                provider_ref("modrinth", "AANobbMI", "mods/sodium-2.jar"),
                // Another content: not disabled because sodium was.
                provider_ref("modrinth", "P7dR8mSH", "mods/fabric-api.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/fabric-api.jar", "mods/sodium-2.jar.disabled"]
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert!(records.contains_key("mods/sodium-2.jar"));
    assert!(!records.contains_key("mods/sodium-1.jar"));
}

/// A direct source without an identity names no content: its file under a new path is a new
/// file, enabled.
#[tokio::test]
async fn should_install_a_direct_file_enabled_when_its_path_is_new() {
    let fixture = Fixture::new().await;
    fixture.put("mods/a-1.jar.disabled", "a 1");
    fixture
        .record(
            "mods/a-1.jar",
            PackFile {
                installed_by: Some(pack_of(&plugin("packwiz"))),
                ..PackFile::from_hash("a-1.jar".into(), sha1("a 1"))
            },
        )
        .await;

    fixture
        .execute(
            existing(vec![hashed("https://cdn.example/a-2.jar", "mods/a-2.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a-2.jar"]);
}

/// A direct file that names its content is what a resolved ref is: a new version of a mod the
/// user disabled stays disabled, and the file is recorded with the identity for updates —
/// while its URL stays the plan origin's word.
#[tokio::test]
async fn should_treat_a_direct_file_as_that_content_when_it_has_an_identity() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    fixture.put("mods/sodium-1.jar.disabled", "sodium 1");
    fixture
        .record(
            "mods/sodium-1.jar",
            PackFile {
                installed_by: Some(pack_of(&modrinth_id())),
                update_provider_id: Some(modrinth_id()),
                update: Some(modrinth_update("AANobbMI", "v1")),
                ..PackFile::from_hash("sodium-1.jar".into(), sha1("sodium 1"))
            },
        )
        .await;
    let url = "https://cdn.modrinth.com/data/AANobbMI/versions/v2/sodium-2.jar";

    fixture
        .execute(
            existing(vec![identified(
                url,
                "mods/sodium-2.jar",
                "modrinth",
                "AANobbMI",
                "v2",
            )]),
            &modrinth_id(),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/sodium-2.jar.disabled"]
    );
    let records = fixture.records(INSTANCE_ID).await;
    let sodium = &records["mods/sodium-2.jar"];
    assert_eq!(sodium.update_provider_id, Some(modrinth_id()));
    let info = &sodium.update.as_ref().unwrap()[&modrinth_id()];
    assert_eq!(
        (info.content_id.as_str(), info.version.as_str()),
        ("AANobbMI", "v2")
    );
    assert_eq!(
        fixture.downloader.downloads()[0].origin,
        modrinth_id().to_string()
    );
}

/// An identity only enriches a direct file: one on a platform no provider serves neither fails
/// the plan nor is recorded.
#[tokio::test]
async fn should_install_a_direct_file_when_no_provider_serves_its_identity() {
    let fixture = Fixture::new().await;
    let url = "https://cdn.example/a.jar";

    fixture
        .execute(
            existing(vec![identified(
                url,
                "mods/a.jar",
                "curseforge",
                "238222",
                "4512345",
            )]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a.jar"]);
    assert!(
        fixture.records(INSTANCE_ID).await["mods/a.jar"]
            .update
            .is_none()
    );
}
