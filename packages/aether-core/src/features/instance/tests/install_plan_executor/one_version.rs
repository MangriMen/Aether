//! An instance never holds two versions of one content.

use crate::features::instance::{FileSide, InstallFileEntry, PackFile, PlanOrigin};

use super::common::*;

const MODMENU: &str = "mOgUt4GM";

/// The plan of Mod Menu installed on its own, as `InstallContentUseCase` names it.
fn modmenu_on_its_own() -> PlanOrigin {
    PlanOrigin {
        provider_id: modrinth_id(),
        content_id: MODMENU.into(),
    }
}

/// Mod Menu's file of `version` at `path`, as the plan of `origin` recorded it.
fn modmenu_record(path: &str, body: &str, version: &str, origin: PlanOrigin) -> PackFile {
    PackFile {
        installed_by: Some(origin),
        update_provider_id: Some(modrinth_id()),
        update: Some(modrinth_update(MODMENU, version)),
        ..PackFile::from_hash(file_name(path), sha1(body))
    }
}

/// Mod Menu's file of `version`, as a `.mrpack` or Modrinth's mod plan lists it.
fn modmenu_entry(version: &str) -> InstallFileEntry {
    identified(
        &format!(
            "https://cdn.modrinth.com/data/{MODMENU}/versions/{version}/modmenu-{version}.jar"
        ),
        &format!("mods/modmenu-{version}.jar"),
        "modrinth",
        MODMENU,
        version,
    )
}

/// A mod installed on its own over the pack's version of it replaces that file — never a second
/// jar next to it — and is the user's from then on.
#[tokio::test]
async fn should_replace_the_pack_version_of_a_mod_when_it_was_installed_on_its_own() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    let pack = pack_of(&modrinth_id());
    fixture.put("mods/modmenu-v7.jar", "modmenu 7");
    fixture
        .record(
            "mods/modmenu-v7.jar",
            modmenu_record("mods/modmenu-v7.jar", "modmenu 7", "v7", pack.clone()),
        )
        .await;
    fixture.put_installed("mods/a.jar", "a", Some(pack)).await;

    fixture
        .execute_as(existing(vec![modmenu_entry("v9")]), modmenu_on_its_own())
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/a.jar", "mods/modmenu-v9.jar"]
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert!(!records.contains_key("mods/modmenu-v7.jar"));
    assert_eq!(
        records["mods/modmenu-v9.jar"].installed_by,
        Some(modmenu_on_its_own())
    );
}

/// The pack's version the user disabled is replaced by one disabled file, as an update of the
/// mod itself leaves it.
#[tokio::test]
async fn should_keep_a_disabled_pack_mod_disabled_when_installed_on_its_own() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    fixture.put("mods/modmenu-v7.jar.disabled", "modmenu 7");
    fixture
        .record(
            "mods/modmenu-v7.jar",
            modmenu_record(
                "mods/modmenu-v7.jar",
                "modmenu 7",
                "v7",
                pack_of(&modrinth_id()),
            ),
        )
        .await;

    fixture
        .execute_as(existing(vec![modmenu_entry("v9")]), modmenu_on_its_own())
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/modmenu-v9.jar.disabled"]
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(records.len(), 1);
    assert_eq!(
        records["mods/modmenu-v9.jar"].installed_by,
        Some(modmenu_on_its_own())
    );
}

/// An update of the pack leaves a mod the user installed on its own as it is: not rolled back to
/// the pack's version, not joined by it, not taken back as the pack's.
#[tokio::test]
async fn should_leave_a_mod_alone_when_it_was_installed_on_its_own_and_a_pack_updates() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    let pack = pack_of(&modrinth_id());
    fixture.put("mods/modmenu-v9.jar", "modmenu 9");
    fixture
        .record(
            "mods/modmenu-v9.jar",
            modmenu_record(
                "mods/modmenu-v9.jar",
                "modmenu 9",
                "v9",
                modmenu_on_its_own(),
            ),
        )
        .await;
    let a = "https://cdn.example/a.jar";

    fixture
        .execute_as(
            existing(vec![modmenu_entry("v8"), hashed(a, "mods/a.jar")]),
            pack.clone(),
        )
        .await
        .unwrap();

    assert_eq!(fixture.downloaded_urls(), [a]);
    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        ["mods/a.jar", "mods/modmenu-v9.jar"]
    );
    let records = fixture.records(INSTANCE_ID).await;
    assert_eq!(
        records["mods/modmenu-v9.jar"].installed_by,
        Some(modmenu_on_its_own())
    );
    assert_eq!(records["mods/a.jar"].installed_by, Some(pack));
}

/// The pack's own copy left next to a mod installed on its own goes with the pack's next update;
/// the user's version stays.
#[tokio::test]
async fn should_remove_the_pack_copy_of_a_mod_when_it_was_installed_on_its_own() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    let pack = pack_of(&modrinth_id());
    fixture.put("mods/modmenu-v7.jar", "modmenu 7");
    fixture
        .record(
            "mods/modmenu-v7.jar",
            modmenu_record("mods/modmenu-v7.jar", "modmenu 7", "v7", pack.clone()),
        )
        .await;
    fixture.put("mods/modmenu-v9.jar", "modmenu 9");
    fixture
        .record(
            "mods/modmenu-v9.jar",
            modmenu_record(
                "mods/modmenu-v9.jar",
                "modmenu 9",
                "v9",
                modmenu_on_its_own(),
            ),
        )
        .await;

    fixture
        .execute_as(existing(vec![modmenu_entry("v7")]), pack)
        .await
        .unwrap();

    assert!(fixture.downloaded_urls().is_empty());
    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/modmenu-v9.jar"]);
    assert_eq!(
        fixture
            .records(INSTANCE_ID)
            .await
            .keys()
            .collect::<Vec<_>>(),
        ["mods/modmenu-v9.jar"]
    );
}

/// Another version is told by the content its record names, never by a look-alike file name: a
/// file without update info — added by hand, an override — is not replaced.
#[tokio::test]
async fn should_not_replace_a_file_when_its_content_is_not_known() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;
    fixture
        .put_installed("mods/modmenu-v7.jar", "by hand", None)
        .await;
    fixture.put("mods/modmenu-v8.jar", "unlisted");

    fixture
        .execute_as(existing(vec![modmenu_entry("v9")]), modmenu_on_its_own())
        .await
        .unwrap();

    assert_eq!(
        fixture.instance_files(INSTANCE_ID),
        [
            "mods/modmenu-v7.jar",
            "mods/modmenu-v8.jar",
            "mods/modmenu-v9.jar"
        ]
    );
}

/// Every planned file is recorded with the side the plan gives it, and with a name: the plan
/// entry's, else the one its ref resolved to; a file nobody names has none.
#[tokio::test]
async fn should_record_the_name_and_side_of_planned_files_when_installing() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;

    fixture
        .execute(
            existing(vec![
                InstallFileEntry {
                    side: FileSide::Client,
                    name: Some("Mod Menu 9".into()),
                    ..modmenu_entry("v9")
                },
                hashed("https://cdn.example/a.jar", "mods/a.jar"),
                provider_ref("modrinth", "AANobbMI", "mods/sodium.jar"),
            ]),
            &modrinth_id(),
        )
        .await
        .unwrap();

    let records = fixture.records(INSTANCE_ID).await;
    let recorded = |path: &str| {
        let record = &records[path];
        (record.name.as_deref(), record.side.as_deref())
    };
    assert_eq!(
        recorded("mods/modmenu-v9.jar"),
        (Some("Mod Menu 9"), Some("client"))
    );
    assert_eq!(recorded("mods/a.jar"), (None, Some("both")));
    assert_eq!(
        recorded("mods/sodium.jar"),
        (Some("AANobbMI newest"), Some("both"))
    );
}

/// A dropped path that differs from a planned one only in case may be the planned file itself
/// where the file system ignores case (Windows, macOS): it is not removed.
#[cfg(any(windows, target_os = "macos"))]
#[tokio::test]
async fn should_not_remove_a_dropped_path_that_differs_only_in_case_when_the_file_system_ignores_case()
 {
    let (fixture, url) = install_over_a_path_that_differs_only_in_case().await;

    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "mods/sodium.jar")
            .as_deref(),
        Some(url)
    );
    assert!(
        fixture
            .records(INSTANCE_ID)
            .await
            .contains_key("mods/Sodium.jar")
    );
}

/// Where the file system tells cases apart (Linux) `mods/Sodium.jar` is another file than the
/// planned `mods/sodium.jar`: it is removed.
#[cfg(not(any(windows, target_os = "macos")))]
#[tokio::test]
async fn should_remove_a_dropped_path_that_differs_only_in_case_when_the_file_system_is_case_sensitive()
 {
    let (fixture, url) = install_over_a_path_that_differs_only_in_case().await;

    assert_eq!(
        fixture
            .instance_file(INSTANCE_ID, "mods/sodium.jar")
            .as_deref(),
        Some(url)
    );
    assert!(
        fixture
            .instance_file(INSTANCE_ID, "mods/Sodium.jar")
            .is_none()
    );
    assert!(
        !fixture
            .records(INSTANCE_ID)
            .await
            .contains_key("mods/Sodium.jar")
    );
}

/// Installs `mods/sodium.jar` over a recorded `mods/Sodium.jar` of the same pack.
async fn install_over_a_path_that_differs_only_in_case() -> (Fixture, &'static str) {
    let fixture = Fixture::new().await;
    fixture
        .put_installed("mods/Sodium.jar", "old", Some(pack_of(&plugin("packwiz"))))
        .await;
    let url = "https://cdn.example/sodium.jar";

    fixture
        .execute(
            existing(vec![hashed(url, "mods/sodium.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    (fixture, url)
}
