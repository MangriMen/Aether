//! Overrides a pack ships as they are.
use crate::features::minecraft::GameVersion;

use crate::features::instance::InstanceName;

use std::{io::Write, path::PathBuf};

use crate::features::{
    instance::{
        AnchoredPath, FileSide, InstallPlan, InstallTarget, InstanceError, InstanceId, NewInstance,
        OverrideLocation, OverrideSource,
    },
    minecraft::ModLoader,
};

use super::common::*;

/// Zip slip: an entry escaping the archive refuses the whole plan before
/// anything is downloaded or written — the entry's subdirectory does not matter.
#[tokio::test]
async fn should_refuse_an_archive_when_an_entry_leads_out_of_it() {
    for evil in [
        "../evil",
        "overrides/../../evil",
        "overrides\\..\\..\\evil",
        "/evil",
        "C:/evil",
        "\\\\server\\evil",
    ] {
        let fixture = Fixture::new().await;
        let archive = zip_archive(
            &fixture,
            &[("overrides/config/a.toml", "a"), (evil, "evil")],
        );

        let err = fixture
            .execute(
                InstallPlan {
                    overrides: archive_overrides(archive),
                    ..existing(vec![direct("https://cdn.example/a.jar", "mods/a.jar")])
                },
                &plugin("packwiz"),
            )
            .await
            .unwrap_err();

        assert!(
            matches!(err, InstanceError::InvalidRelativePath { .. }),
            "{evil}: {err:?}"
        );
        assert!(fixture.downloader.downloads().is_empty(), "{evil}");
        assert!(fixture.instance_files(INSTANCE_ID).is_empty(), "{evil}");
    }
}

/// An entry inside the subdirectory that stays in the archive but not in the instance once the
/// subdirectory is stripped (`overrides/../evil`) is refused by the content path rules.
#[tokio::test]
async fn should_refuse_an_override_when_it_leads_out_of_the_instance() {
    let fixture = Fixture::new().await;
    let archive = zip_archive(&fixture, &[("overrides/../evil", "evil")]);

    let err = fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::InvalidRelativePath { .. }),
        "{err:?}"
    );
    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
}

#[tokio::test]
async fn should_refuse_an_override_when_it_leads_into_the_instance_metadata() {
    let fixture = Fixture::new().await;
    let archive = zip_archive(&fixture, &[("overrides/.metadata/instance.json", "{}")]);

    let err = fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![direct("https://cdn.example/a.jar", "mods/a.jar")])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::InvalidRelativePath { .. }),
        "{err:?}"
    );
    assert!(fixture.downloader.downloads().is_empty());
}

/// A stored zip in the cache directory whose every entry holds `size` bytes but declares
/// `declared`, as a zip bomb's headers do; the data and its checksum are left whole.
fn understated_archive(fixture: &Fixture, names: &[&str], size: usize, declared: u32) -> PathBuf {
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for name in names {
        writer.start_file(*name, options).unwrap();
        writer.write_all(&vec![b'x'; size]).unwrap();
    }
    let mut bytes = writer.finish().unwrap().into_inner();

    // The uncompressed size in each local and central directory header.
    for (signature, offset) in [(b"PK\x03\x04", 22), (b"PK\x01\x02", 24)] {
        let starts: Vec<usize> = (0..bytes.len() - 4)
            .filter(|&start| &bytes[start..start + 4] == signature)
            .collect();
        for start in starts {
            bytes[start + offset..start + offset + 4].copy_from_slice(&declared.to_le_bytes());
        }
    }

    let dir = fixture.location_info.cache_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bomb.mrpack");
    std::fs::write(&path, bytes).unwrap();
    path
}

/// The number of override files is bounded for the plan, over its sources together — each
/// source here fits, the two do not — and refused before anything is downloaded. A source for
/// the server is not listed, so it counts for nothing.
#[tokio::test]
async fn should_refuse_overrides_when_all_sources_together_pass_the_file_limit() {
    let entries = [
        ("overrides/a.toml", "a"),
        ("overrides/b.toml", "b"),
        ("client-overrides/c.toml", "c"),
        ("server-overrides/d.toml", "d"),
    ];
    let sources = |archive: PathBuf| {
        vec![
            archive_subdir(archive.clone(), "overrides", FileSide::Both),
            archive_subdir(archive.clone(), "client-overrides", FileSide::Client),
            archive_subdir(archive, "server-overrides", FileSide::Server),
        ]
    };

    let mut fixture = Fixture::new().await;
    fixture.override_limits = Some((2, u64::MAX));
    let archive = zip_archive(&fixture, &entries);

    let err = fixture
        .execute(
            InstallPlan {
                overrides: sources(archive),
                ..existing(vec![direct("https://cdn.example/a.jar", "mods/a.jar")])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::ContentProviderError { .. }),
        "{err:?}"
    );
    assert!(fixture.downloader.downloads().is_empty());
    assert!(fixture.instance_files(INSTANCE_ID).is_empty());

    let mut fixture = Fixture::new().await;
    fixture.override_limits = Some((3, u64::MAX));
    let archive = zip_archive(&fixture, &entries);

    fixture
        .execute(
            InstallPlan {
                overrides: sources(archive),
                ..existing(vec![])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["a.toml", "b.toml", "c.toml"]
    );
}

/// An archive too large by its own headers is refused before anything is downloaded.
#[tokio::test]
async fn should_refuse_overrides_when_they_declare_more_bytes_than_the_limit() {
    let mut fixture = Fixture::new().await;
    fixture.override_limits = Some((usize::MAX, 150));
    let archive = understated_archive(&fixture, &["overrides/a.txt", "overrides/b.txt"], 100, 100);

    let err = fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![direct("https://cdn.example/a.jar", "mods/a.jar")])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::FileTooLarge { path, size: 200, limit: 150 } if path == "b.txt"),
        "{err:?}"
    );
    assert!(fixture.downloader.downloads().is_empty());
    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
}

/// An archive whose headers understate its entries passes the early check and is refused by the
/// bytes it unpacks to, while staging: nothing of the plan reaches the instance — neither the
/// overrides nor the planned file downloaded before them — and nothing is recorded.
#[tokio::test]
async fn should_refuse_an_archive_when_it_unpacks_to_more_than_it_declares() {
    let mut fixture = Fixture::new().await;
    fixture.override_limits = Some((usize::MAX, 150));
    let archive = understated_archive(&fixture, &["overrides/a.txt", "overrides/b.txt"], 100, 10);

    let err = fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![direct("https://cdn.example/a.jar", "mods/a.jar")])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(
            &err,
            InstanceError::Storage(crate::features::instance::StorageError::Io(io))
                if io.kind() == std::io::ErrorKind::InvalidData
        ),
        "{err:?}"
    );
    assert_eq!(fixture.downloaded_urls(), ["https://cdn.example/a.jar"]);
    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
    assert!(fixture.records(INSTANCE_ID).await.is_empty());
}

/// Only the subdirectory is applied, stripped of its name, with `/`-separated paths; directory
/// entries create nothing on their own, and the overrides are not recorded as the pack's.
#[tokio::test]
async fn should_apply_the_overrides_subdirectory_when_an_archive_has_one() {
    let fixture = Fixture::new().await;
    fixture.put("config/a.toml", "edited by the user");
    let archive = zip_archive(
        &fixture,
        &[
            ("modrinth.index.json", "{}"),
            ("overrides/", ""),
            ("overrides/config/", ""),
            ("overrides/config/a.toml", "pack a"),
            ("overrides/config/sub/b.json", "pack b"),
            ("overrides/options.txt", "options"),
            ("client-overrides/config/c.toml", "client"),
        ],
    );

    fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![hashed("https://cdn.example/a.jar", "mods/a.jar")])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        [
            "config/a.toml",
            "config/sub/b.json",
            "mods/a.jar",
            "options.txt"
        ]
    );
    // The pack's config wins over the user's edit of it.
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "config/a.toml").unwrap(),
        "pack a"
    );
    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "config/sub/b.json")
            .unwrap(),
        "pack b"
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records.keys().collect::<Vec<_>>(), ["mods/a.jar"]);
    assert!(fixture.staging_is_clean());
}

/// A `.mrpack`'s sides: `client-overrides/` is laid over `overrides/` — a path in both gets the
/// client's file, compared ignoring case — while `server-overrides/` is left out. A planned file
/// still wins over every source.
#[tokio::test]
async fn should_layer_client_overrides_and_leave_out_server_ones_when_both_exist() {
    let fixture = Fixture::new().await;
    let archive = zip_archive(
        &fixture,
        &[
            ("overrides/config/a.toml", "common a"),
            ("overrides/config/B.toml", "common b"),
            ("overrides/options.txt", "common options"),
            ("overrides/mods/a.jar", "override a"),
            ("client-overrides/config/a.toml", "client a"),
            ("client-overrides/config/b.toml", "client b"),
            ("client-overrides/mods/a.jar", "client override a"),
            ("server-overrides/options.txt", "server options"),
            ("server-overrides/server.properties", "server"),
        ],
    );

    fixture
        .execute(
            InstallPlan {
                overrides: ["overrides", "client-overrides", "server-overrides"]
                    .into_iter()
                    .zip([FileSide::Both, FileSide::Client, FileSide::Server])
                    .map(|(subdir, side)| archive_subdir(archive.clone(), subdir, side))
                    .collect(),
                ..existing(vec![hashed("https://cdn.example/a.jar", "mods/a.jar")])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        [
            "config/a.toml",
            "config/b.toml",
            "mods/a.jar",
            "options.txt"
        ]
    );
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "config/a.toml").unwrap(),
        "client a"
    );
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "config/b.toml").unwrap(),
        "client b"
    );
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "options.txt").unwrap(),
        "common options"
    );
    assert_ne!(
        fixture.instance_file(INSTANCE_ID, "mods/a.jar").unwrap(),
        "client override a"
    );
    assert!(fixture.staging_is_clean());
}

#[tokio::test]
async fn should_copy_an_overrides_directory_recursively_when_it_is_nested() {
    let fixture = Fixture::new().await;
    let source = fixture.location_info.cache_dir().join("plugin-overrides");
    std::fs::create_dir_all(source.join("config/sub")).unwrap();
    std::fs::create_dir_all(source.join("empty")).unwrap();
    std::fs::write(source.join("config/sub/b.json"), "pack b").unwrap();
    std::fs::write(source.join("options.txt"), "options").unwrap();

    fixture
        .execute(
            InstallPlan {
                overrides: vec![OverrideSource::new(
                    OverrideLocation::Directory {
                        path: AnchoredPath::at_parent(&source),
                    },
                    FileSide::Both,
                )],
                ..existing(vec![])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["config/sub/b.json", "options.txt"]
    );
    // Copied, not moved: the source is the plugin's.
    assert!(source.join("options.txt").exists());
}

#[tokio::test]
async fn should_apply_overrides_when_the_instance_is_new() {
    let fixture = Fixture::new().await;
    let archive = zip_archive(&fixture, &[("overrides/options.txt", "options")]);

    let instance_id = fixture
        .execute(
            InstallPlan {
                target: InstallTarget::New(NewInstance {
                    name: InstanceName::parse("Pack").unwrap(),
                    game_version: GameVersion::from_static("1.21.1"),
                    mod_loader: ModLoader::Fabric,
                    loader_version: None,
                    icon_path: None,
                    skip_install_instance: Some(true),
                    pack_info: None,
                }),
                files: vec![],
                overrides: archive_overrides(archive),
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_file(&instance_id, "options.txt").unwrap(),
        "options"
    );
}

/// The order: planned files win over overrides at their paths — enabled, disabled, or spelled
/// in another case — and an override at a path the plan would drop keeps it: the file is written
/// anew and stays recorded as the pack's.
#[tokio::test]
async fn should_apply_overrides_after_planned_files_when_both_name_a_path() {
    let fixture = Fixture::new().await;
    let owner = Some(pack_of(&plugin("packwiz")));
    fixture
        .put_installed("mods/dropped.jar", "dropped", owner)
        .await;
    fixture.put("mods/b.jar.disabled", "b old");
    let archive = zip_archive(
        &fixture,
        &[
            ("overrides/mods/a.jar", "override a"),
            ("overrides/mods/B.jar", "override b"),
            ("overrides/mods/dropped.jar", "override dropped"),
        ],
    );

    fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![
                    hashed("https://cdn.example/a.jar", "mods/a.jar"),
                    hashed("https://cdn.example/b.jar", "mods/b.jar"),
                ])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/a.jar", "mods/b.jar.disabled", "mods/dropped.jar"]
    );
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "mods/a.jar").unwrap(),
        "https://cdn.example/a.jar"
    );
    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "mods/b.jar.disabled")
            .unwrap(),
        "https://cdn.example/b.jar"
    );
    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "mods/dropped.jar")
            .unwrap(),
        "override dropped"
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(
        records["mods/dropped.jar"].installed_by,
        Some(pack_of(&plugin("packwiz")))
    );
    assert_eq!(
        records["mods/dropped.jar"].hash,
        sha1_hash("override dropped")
    );
}

/// An override at a path the user disabled replaces the disabled file, never lands next to it.
#[tokio::test]
async fn should_keep_a_disabled_override_disabled_when_the_plan_runs_again() {
    let fixture = Fixture::new().await;
    fixture.put("mods/extra.jar.disabled", "old");
    let archive = zip_archive(&fixture, &[("overrides/mods/extra.jar", "new")]);

    for _ in 0..2 {
        fixture
            .execute(
                InstallPlan {
                    overrides: archive_overrides(archive.clone()),
                    ..existing(vec![])
                },
                &plugin("packwiz"),
            )
            .await
            .unwrap();

        assert_eq!(
            fixture.instance_files(INSTANCE_ID),
            ["mods/extra.jar.disabled"]
        );
        assert_eq!(
            fixture
                .instance_file(INSTANCE_ID, "mods/extra.jar.disabled")
                .unwrap(),
            "new"
        );
    }
}

/// Overrides are copied out before the instance is touched: a failed download leaves no
/// override in it either.
#[tokio::test]
async fn should_apply_no_overrides_when_a_download_fails() {
    let fixture = Fixture::new().await;
    let archive = zip_archive(&fixture, &[("overrides/options.txt", "options")]);

    fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![direct("https://cdn.example/fail.jar", "mods/a.jar")])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
    assert!(fixture.staging_is_clean());
}

/// Run the plan of `plugin("packwiz")` with `entries` as its overrides.
async fn run_overrides(fixture: &Fixture, entries: &[(&str, &str)]) {
    let archive = zip_archive(fixture, entries);
    fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();
}

/// The bug: a jar an update renames (`a-1.jar` to `a-2.jar`) is the pack's, so the old one goes
/// instead of staying next to the new one — two versions of a mod crash the game.
#[tokio::test]
async fn should_remove_the_old_override_jar_when_an_update_renames_it() {
    let fixture = Fixture::new().await;
    run_overrides(&fixture, &[("overrides/mods/a-1.jar", "one")]).await;
    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a-1.jar"]);

    run_overrides(&fixture, &[("overrides/mods/a-2.jar", "two")]).await;

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a-2.jar"]);
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records.keys().collect::<Vec<_>>(), ["mods/a-2.jar"]);
}

/// An override the pack no longer ships is removed with its record.
#[tokio::test]
async fn should_remove_an_override_jar_when_the_pack_drops_it() {
    let fixture = Fixture::new().await;
    run_overrides(
        &fixture,
        &[("overrides/mods/a.jar", "a"), ("overrides/mods/b.jar", "b")],
    )
    .await;

    run_overrides(&fixture, &[("overrides/mods/a.jar", "a")]).await;

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a.jar"]);
    assert_eq!(fixture.records(INSTANCE_ID).await.len(), 1);
}

/// An override is recorded as a planned file is: the pack's, with the hash of the file and the
/// side of the folder it came from.
#[tokio::test]
async fn should_record_an_override_jar_as_the_packs_when_it_is_installed() {
    let fixture = Fixture::new().await;
    let archive = zip_archive(
        &fixture,
        &[
            ("overrides/mods/a.jar", "a"),
            ("client-overrides/mods/c.jar", "c"),
        ],
    );
    fixture
        .execute(
            InstallPlan {
                overrides: vec![
                    archive_subdir(&archive, "overrides", FileSide::Both),
                    archive_subdir(&archive, "client-overrides", FileSide::Client),
                ],
                ..existing(vec![])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    let records = fixture.records(INSTANCE_ID).await;
    let origin = Some(pack_of(&plugin("packwiz")));
    assert_eq!(records["mods/a.jar"].installed_by, origin);
    assert_eq!(records["mods/a.jar"].hash, sha1_hash("a"));
    assert_eq!(records["mods/a.jar"].side, Some(FileSide::Both));
    assert_eq!(records["mods/c.jar"].installed_by, origin);
    assert_eq!(records["mods/c.jar"].side, Some(FileSide::Client));
}

/// Only a file of a content folder is the pack's to remove: a config, a nested file and a
/// disabled override stay when the pack drops them, and none is recorded.
#[tokio::test]
async fn should_not_record_an_override_when_it_is_not_a_content_file() {
    let fixture = Fixture::new().await;
    let files = [
        ("overrides/config/a.toml", "a"),
        ("overrides/mods/sub/b.jar", "b"),
        ("overrides/mods/c.jar.disabled", "c"),
        ("overrides/options.txt", "o"),
    ];
    run_overrides(&fixture, &files).await;

    assert!(fixture.records(INSTANCE_ID).await.is_empty());

    run_overrides(&fixture, &[("overrides/mods/d.jar", "d")]).await;

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        [
            "config/a.toml",
            "mods/c.jar.disabled",
            "mods/d.jar",
            "mods/sub/b.jar",
            "options.txt"
        ]
    );
}

/// A jar the user disabled stays disabled when the pack's override replaces it, and it is
/// still the pack's: recorded under its enabled path, and removed disabled when dropped.
#[tokio::test]
async fn should_keep_the_record_of_a_disabled_override_jar_when_it_is_replaced_and_dropped() {
    let fixture = Fixture::new().await;
    run_overrides(&fixture, &[("overrides/mods/a.jar", "one")]).await;
    let dir = fixture
        .location_info
        .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap());
    std::fs::rename(dir.join("mods/a.jar"), dir.join("mods/a.jar.disabled")).unwrap();

    run_overrides(&fixture, &[("overrides/mods/a.jar", "two")]).await;

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a.jar.disabled"]);
    assert_eq!(
        fixture.records(INSTANCE_ID).await["mods/a.jar"].hash,
        sha1_hash("two")
    );

    run_overrides(&fixture, &[]).await;

    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
    assert!(fixture.records(INSTANCE_ID).await.is_empty());
}

/// A file another plan installed stays that plan's: the override over it is left out.
#[tokio::test]
async fn should_leave_out_an_override_when_another_plan_installed_the_file() {
    let fixture = Fixture::new().await;
    let other = pack_of(&plugin("other"));
    fixture
        .put_installed("mods/a.jar", "other's", Some(other.clone()))
        .await;

    run_overrides(&fixture, &[("overrides/mods/a.jar", "pack's")]).await;

    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "mods/a.jar").unwrap(),
        "other's"
    );
    assert_eq!(
        fixture.records(INSTANCE_ID).await["mods/a.jar"].installed_by,
        Some(other)
    );
}

/// A file found on disk without an owner is taken over by an override at its path.
#[tokio::test]
async fn should_take_over_an_unowned_file_when_an_override_names_its_path() {
    let fixture = Fixture::new().await;
    fixture.put_installed("mods/a.jar", "by hand", None).await;

    run_overrides(&fixture, &[("overrides/mods/a.jar", "pack's")]).await;

    let record = &fixture.records(INSTANCE_ID).await["mods/a.jar"];
    assert_eq!(record.installed_by, Some(pack_of(&plugin("packwiz"))));
    assert_eq!(record.hash, sha1_hash("pack's"));
}

/// A planned file the pack turns into an override of a path that is no content file loses its
/// record, and the file stays instead of being removed as dropped.
#[tokio::test]
async fn should_unrecord_a_file_when_the_pack_ships_it_as_an_override_instead() {
    let fixture = Fixture::new().await;
    fixture
        .put_installed("config/a.toml", "old", Some(pack_of(&plugin("packwiz"))))
        .await;

    run_overrides(&fixture, &[("overrides/config/a.toml", "new")]).await;

    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "config/a.toml").unwrap(),
        "new"
    );
    assert!(fixture.records(INSTANCE_ID).await.is_empty());
}

/// A planned file takes over a path the pack's override was at: the old file is replaced, the
/// record is the planned file's.
#[tokio::test]
async fn should_record_the_planned_file_when_the_pack_moves_an_override_jar_to_the_plan() {
    let fixture = Fixture::new().await;
    run_overrides(&fixture, &[("overrides/mods/a.jar", "override")]).await;

    fixture
        .execute(
            existing(vec![hashed("https://cdn.example/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "mods/a.jar").unwrap(),
        "https://cdn.example/a.jar"
    );
    assert_eq!(fixture.records(INSTANCE_ID).await.len(), 1);
}

/// The pack ships a jar it had enabled as disabled now: the enabled file goes, the disabled one
/// is the only copy, and no record is left for it.
#[tokio::test]
async fn should_remove_the_enabled_jar_when_the_pack_ships_it_disabled_instead() {
    let fixture = Fixture::new().await;
    run_overrides(&fixture, &[("overrides/mods/a.jar", "one")]).await;

    run_overrides(&fixture, &[("overrides/mods/a.jar.disabled", "two")]).await;

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a.jar.disabled"]);
    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "mods/a.jar.disabled")
            .unwrap(),
        "two"
    );
    assert!(fixture.records(INSTANCE_ID).await.is_empty());
}

/// A path is claimed once by a source, enabled or disabled: the two are one file.
#[tokio::test]
async fn should_refuse_a_source_when_it_has_a_jar_both_enabled_and_disabled() {
    let fixture = Fixture::new().await;
    let archive = zip_archive(
        &fixture,
        &[
            ("overrides/mods/a.jar", "a"),
            ("overrides/mods/a.jar.disabled", "a"),
        ],
    );

    let err = fixture
        .execute(
            InstallPlan {
                overrides: archive_overrides(archive),
                ..existing(vec![])
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::ContentDuplication { .. }),
        "{err:?}"
    );
}

/// An override that differs from the pack's recorded jar only in case is that very file where
/// the file system ignores case (Windows, macOS): one file, one record, none left stale.
#[cfg(any(windows, target_os = "macos"))]
#[tokio::test]
async fn should_keep_one_file_and_one_record_when_an_override_differs_only_in_case() {
    let fixture = Fixture::new().await;
    run_overrides(&fixture, &[("overrides/mods/a.jar", "one")]).await;

    run_overrides(&fixture, &[("overrides/mods/A.jar", "two")]).await;

    assert_eq!(fixture.instance_files(INSTANCE_ID).len(), 1);
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records.len(), 1);
    assert_eq!(records.values().next().unwrap().hash, sha1_hash("two"));

    run_overrides(&fixture, &[]).await;

    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
    assert!(fixture.records(INSTANCE_ID).await.is_empty());
}

/// Where the file system tells cases apart (Linux) `mods/A.jar` is another file than the
/// recorded `mods/a.jar`: the old one is dropped.
#[cfg(not(any(windows, target_os = "macos")))]
#[tokio::test]
async fn should_replace_the_jar_when_an_override_differs_only_in_case_on_a_case_sensitive_system() {
    let fixture = Fixture::new().await;
    run_overrides(&fixture, &[("overrides/mods/a.jar", "one")]).await;

    run_overrides(&fixture, &[("overrides/mods/A.jar", "two")]).await;

    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/A.jar"]);
    assert_eq!(
        fixture
            .records(INSTANCE_ID)
            .await
            .keys()
            .collect::<Vec<_>>(),
        ["mods/A.jar"]
    );
}
