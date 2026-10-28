use std::sync::Arc;

use tempfile::TempDir;

use crate::features::instance::{
    InstanceError, InstanceId, app::ContentFileService, infra::FsContentFileService,
};
use crate::features::settings::LocationInfo;
use crate::test_support::link_dir;

fn setup_temp_location() -> (TempDir, Arc<LocationInfo>) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let location_info = Arc::new(LocationInfo::new(
        temp_dir.path().join("settings"),
        temp_dir.path().to_path_buf(),
    ));
    (temp_dir, location_info)
}

async fn create_instance_dir(location_info: &LocationInfo, instance_id: &InstanceId) {
    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");
    tokio::fs::create_dir_all(dir.join("config"))
        .await
        .expect("Failed to create config dir");

    tokio::fs::write(dir.join("mods").join("test.jar"), b"data")
        .await
        .unwrap();
    tokio::fs::write(dir.join("mods").join("another.jar"), b"more data")
        .await
        .unwrap();
    tokio::fs::write(dir.join("config").join("example.toml"), b"config")
        .await
        .unwrap();
}

#[tokio::test]
async fn should_remove_a_file_when_one_file_is_given() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-1").unwrap();

    create_instance_dir(&location_info, instance_id).await;

    let file_path = location_info
        .instance_dir(instance_id)
        .join("mods/test.jar");
    assert!(file_path.exists(), "File should exist before removal");

    // Act
    service
        .remove_content_files(instance_id, &["mods/test.jar".to_string()])
        .await
        .expect("remove_content_files should succeed");

    // Assert
    assert!(!file_path.exists(), "File should be removed");
    assert!(
        location_info
            .instance_dir(instance_id)
            .join("mods/another.jar")
            .exists(),
        "Other files should still exist"
    );
}

#[tokio::test]
async fn should_remove_files_when_several_are_given() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-2").unwrap();

    create_instance_dir(&location_info, instance_id).await;

    let paths = vec!["mods/test.jar".to_string(), "mods/another.jar".to_string()];

    // Act
    service
        .remove_content_files(instance_id, &paths)
        .await
        .expect("remove_content_files should succeed");

    // Assert
    let instance_dir = location_info.instance_dir(instance_id);
    for path in &paths {
        assert!(
            !instance_dir.join(path).exists(),
            "File {path} should be removed"
        );
    }
    assert!(
        instance_dir.join("config/example.toml").exists(),
        "Unrelated files should still exist"
    );
}

#[tokio::test]
async fn should_remove_a_disabled_file_when_the_file_is_disabled() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-3").unwrap();

    let instance_dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(instance_dir.join("mods"))
        .await
        .expect("Failed to create dirs");

    // Create a .disabled file instead of the regular one
    tokio::fs::write(instance_dir.join("mods/test.jar.disabled"), b"data")
        .await
        .unwrap();

    // Act — should find and remove the .disabled variant
    service
        .remove_content_files(instance_id, &["mods/test.jar".to_string()])
        .await
        .expect("remove_content_files should succeed");

    // Assert
    assert!(
        !instance_dir.join("mods/test.jar.disabled").exists(),
        "Disabled file should be removed"
    );
}

#[tokio::test]
async fn should_succeed_when_the_file_to_remove_does_not_exist() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-4").unwrap();

    // No files created at all — should not fail
    service
        .remove_content_files(instance_id, &["mods/nonexistent.jar".to_string()])
        .await
        .expect("remove_content_files should succeed for non-existent files");

    assert!(
        !location_info.instance_dir(instance_id).exists(),
        "removing a missing file must not create the instance directory"
    );
}

/// A directory at the content path cannot be removed as a file on any OS: it stands for a file
/// the user's system holds on to.
#[tokio::test]
async fn should_fail_with_the_file_kept_when_it_cannot_be_removed() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-held").unwrap();
    let instance_dir = location_info.instance_dir(instance_id);
    std::fs::create_dir_all(instance_dir.join("mods/held.jar")).unwrap();

    let err = service
        .remove_content_files(instance_id, &["mods/held.jar".to_string()])
        .await
        .unwrap_err();

    assert!(matches!(err, InstanceError::Storage(_)), "{err:?}");
    assert!(instance_dir.join("mods/held.jar").exists());
}

// ─── Moving a file out ─────────────────────────────────────────

#[tokio::test]
async fn should_move_a_file_out_and_back_when_it_is_there() {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-move").unwrap();
    create_instance_dir(&location_info, instance_id).await;
    let backup = temp_dir.path().join("backup");

    let moved = service
        .move_content_file_out(instance_id, "mods/test.jar", &backup)
        .await
        .unwrap();

    assert!(moved);
    let file = location_info
        .instance_dir(instance_id)
        .join("mods/test.jar");
    assert!(!file.exists());
    assert_eq!(std::fs::read(&backup).unwrap(), b"data");

    service
        .install_content_file(instance_id, "mods/test.jar", &backup)
        .await
        .unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), b"data");
}

/// Only the exact path: a disabled file is not taken for the enabled one.
#[tokio::test]
async fn should_move_nothing_out_when_the_exact_path_is_empty() {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-move-none").unwrap();
    let dir = location_info.instance_dir(instance_id);
    std::fs::create_dir_all(dir.join("mods")).unwrap();
    std::fs::write(dir.join("mods/a.jar.disabled"), b"data").unwrap();

    let moved = service
        .move_content_file_out(instance_id, "mods/a.jar", &temp_dir.path().join("backup"))
        .await
        .unwrap();

    assert!(!moved);
    assert!(dir.join("mods/a.jar.disabled").exists());
}

#[tokio::test]
async fn should_refuse_to_move_out_a_directory_when_it_sits_at_a_content_path() {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-move-dir").unwrap();
    let dir = location_info.instance_dir(instance_id);
    std::fs::create_dir_all(dir.join("mods/a.jar")).unwrap();

    let err = service
        .move_content_file_out(instance_id, "mods/a.jar", &temp_dir.path().join("backup"))
        .await
        .unwrap_err();

    assert!(matches!(err, InstanceError::Storage(_)), "{err:?}");
    assert!(dir.join("mods/a.jar").is_dir());
}

#[tokio::test]
async fn should_refuse_to_move_out_a_path_when_it_leads_out_of_the_instance() {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-instance-move-escape").unwrap();
    create_instance_dir(&location_info, instance_id).await;
    std::fs::write(temp_dir.path().join("escaped.jar"), b"outside").unwrap();

    let result = service
        .move_content_file_out(
            instance_id,
            "../../escaped.jar",
            &temp_dir.path().join("backup"),
        )
        .await;

    assert!(
        matches!(result, Err(InstanceError::InvalidRelativePath { .. })),
        "{result:?}"
    );
    assert!(temp_dir.path().join("escaped.jar").exists());
    assert!(!temp_dir.path().join("backup").exists());
}

// ─── Enable / Disable tests ────────────────────────────────────

#[tokio::test]
async fn should_enable_a_file_when_it_is_disabled() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-enable-1").unwrap();

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");

    // Create disabled file
    tokio::fs::write(dir.join("mods/test.jar.disabled"), b"data")
        .await
        .unwrap();

    assert!(
        dir.join("mods/test.jar.disabled").exists(),
        "Disabled file should exist"
    );
    assert!(
        !dir.join("mods/test.jar").exists(),
        "Enabled file should not exist yet"
    );

    // Act
    service
        .enable_content_files(instance_id, &["mods/test.jar".to_string()])
        .await
        .expect("enable_content_files should succeed");

    // Assert
    assert!(dir.join("mods/test.jar").exists(), "File should be enabled");
    assert!(
        !dir.join("mods/test.jar.disabled").exists(),
        "Disabled file should be removed"
    );
}

#[tokio::test]
async fn should_disable_a_file_when_it_is_enabled() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-disable-1").unwrap();

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");

    tokio::fs::write(dir.join("mods/test.jar"), b"data")
        .await
        .unwrap();

    assert!(
        dir.join("mods/test.jar").exists(),
        "Enabled file should exist"
    );

    // Act
    service
        .disable_content_files(instance_id, &["mods/test.jar".to_string()])
        .await
        .expect("disable_content_files should succeed");

    // Assert
    assert!(
        dir.join("mods/test.jar.disabled").exists(),
        "Disabled file should exist"
    );
    assert!(
        !dir.join("mods/test.jar").exists(),
        "Original file should be removed"
    );
}

#[tokio::test]
async fn should_stay_enabled_when_a_file_is_already_enabled() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-enable-idempotent").unwrap();

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");

    tokio::fs::write(dir.join("mods/test.jar"), b"data")
        .await
        .unwrap();

    // Act — should do nothing and not fail
    service
        .enable_content_files(instance_id, &["mods/test.jar".to_string()])
        .await
        .expect("enable should succeed even if already enabled");

    assert!(
        dir.join("mods/test.jar").exists(),
        "File should still exist"
    );
}

#[tokio::test]
async fn should_stay_disabled_when_a_file_is_already_disabled() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-disable-idempotent").unwrap();

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");

    tokio::fs::write(dir.join("mods/test.jar.disabled"), b"data")
        .await
        .unwrap();

    // Act — should do nothing and not fail
    service
        .disable_content_files(instance_id, &["mods/test.jar".to_string()])
        .await
        .expect("disable should succeed even if already disabled");

    assert!(
        dir.join("mods/test.jar.disabled").exists(),
        "Disabled file should still exist"
    );
}

#[tokio::test]
async fn should_enable_files_when_several_are_given() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-enable-multi").unwrap();

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");

    tokio::fs::write(dir.join("mods/a.jar.disabled"), b"data")
        .await
        .unwrap();
    tokio::fs::write(dir.join("mods/b.jar.disabled"), b"data")
        .await
        .unwrap();

    // Act
    service
        .enable_content_files(
            instance_id,
            &["mods/a.jar".to_string(), "mods/b.jar".to_string()],
        )
        .await
        .expect("enable_content_files multiple should succeed");

    assert!(dir.join("mods/a.jar").exists(), "a.jar should be enabled");
    assert!(dir.join("mods/b.jar").exists(), "b.jar should be enabled");
    assert!(
        !dir.join("mods/a.jar.disabled").exists(),
        "a.jar.disabled should not exist"
    );
    assert!(
        !dir.join("mods/b.jar.disabled").exists(),
        "b.jar.disabled should not exist"
    );
}

#[tokio::test]
async fn should_disable_files_when_several_are_given() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-disable-multi").unwrap();

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");

    tokio::fs::write(dir.join("mods/a.jar"), b"data")
        .await
        .unwrap();
    tokio::fs::write(dir.join("mods/b.jar"), b"data")
        .await
        .unwrap();

    // Act
    service
        .disable_content_files(
            instance_id,
            &["mods/a.jar".to_string(), "mods/b.jar".to_string()],
        )
        .await
        .expect("disable_content_files multiple should succeed");

    assert!(
        dir.join("mods/a.jar.disabled").exists(),
        "a.jar should be disabled"
    );
    assert!(
        dir.join("mods/b.jar.disabled").exists(),
        "b.jar should be disabled"
    );
    assert!(!dir.join("mods/a.jar").exists(), "a.jar should not exist");
    assert!(!dir.join("mods/b.jar").exists(), "b.jar should not exist");
}

#[tokio::test]
async fn should_succeed_when_the_file_to_enable_or_disable_does_not_exist() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-nonexistent-toggle").unwrap();

    // No files at all — should not fail
    service
        .enable_content_files(instance_id, &["mods/nonexistent.jar".to_string()])
        .await
        .expect("enable should be idempotent for missing files");

    service
        .disable_content_files(instance_id, &["mods/nonexistent.jar".to_string()])
        .await
        .expect("disable should be idempotent for missing files");
}

// ── install_content_file: where a file may go ──

async fn temp_file(location_info: &LocationInfo, body: &[u8]) -> std::path::PathBuf {
    let dir = location_info.temp_dir();
    tokio::fs::create_dir_all(&dir).await.unwrap();
    let path = dir.join(uuid::Uuid::new_v4().to_string());
    tokio::fs::write(&path, body).await.unwrap();
    path
}

#[tokio::test]
async fn should_install_into_a_new_subdirectory_when_it_does_not_exist_yet() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-install").unwrap();
    tokio::fs::create_dir_all(location_info.instance_dir(instance_id))
        .await
        .unwrap();
    let temp = temp_file(&location_info, b"jar").await;

    service
        .install_content_file(instance_id, "config/sodium/options.json", &temp)
        .await
        .expect("a plain path should be installed");

    assert_eq!(
        std::fs::read(
            location_info
                .instance_dir(instance_id)
                .join("config/sodium/options.json")
        )
        .unwrap(),
        b"jar"
    );
    assert!(!temp.exists(), "the temp file is moved, not copied");
}

/// A content path may come from a provider's plan: one that leaves the instance, or reaches
/// into the launcher's `.metadata`, is refused and nothing is moved.
#[tokio::test]
async fn should_refuse_a_path_when_it_leads_out_of_the_content_area() {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-install-refused").unwrap();
    tokio::fs::create_dir_all(location_info.instance_dir(instance_id))
        .await
        .unwrap();

    for path in [
        "../escaped.jar",
        "mods/../../escaped.jar",
        "/escaped.jar",
        "C:/escaped.jar",
        ".metadata/instance.json",
        ".Metadata/plugins/packwiz/packwiz.toml",
    ] {
        let temp = temp_file(&location_info, b"jar").await;

        let err = service
            .install_content_file(instance_id, path, &temp)
            .await
            .expect_err(path);

        assert!(
            matches!(err, InstanceError::InvalidRelativePath { .. }),
            "{path}: {err:?}"
        );
        assert!(
            service.check_content_path(path).is_err(),
            "{path}: the check alone refuses it too"
        );
        assert!(temp.exists(), "{path}: nothing is moved");
    }

    assert!(!temp_dir.path().join("escaped.jar").exists());
    assert!(!location_info.instances_dir().join("escaped.jar").exists());
    assert!(service.check_content_path("mods/sodium.jar").is_ok());
}

#[tokio::test]
async fn should_refuse_an_install_when_a_symlink_leads_out_of_the_instance() {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-install-symlink").unwrap();
    let instance_dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(&instance_dir).await.unwrap();
    let outside = temp_dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();

    link_dir(&outside, &instance_dir.join("mods"));

    let temp = temp_file(&location_info, b"jar").await;
    let err = service
        .install_content_file(instance_id, "mods/escaped.jar", &temp)
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::PathEscapesInstance { .. }),
        "{err:?}"
    );
    assert!(!outside.join("escaped.jar").exists());
}

// ── remove / enable / disable: where a file may be changed ──

/// Each operation over `paths`, as a name and its result.
async fn change_all(
    service: &FsContentFileService,
    instance_id: &InstanceId,
    paths: &[String],
) -> Vec<(&'static str, Result<(), InstanceError>)> {
    vec![
        (
            "remove",
            service.remove_content_files(instance_id, paths).await,
        ),
        (
            "enable",
            service.enable_content_files(instance_id, paths).await,
        ),
        (
            "disable",
            service.disable_content_files(instance_id, paths).await,
        ),
    ]
}

/// The paths come from the pack storage and the UI: one out of the instance, or into the
/// launcher's `.metadata`, is skipped by every operation — nothing is done to it — while the
/// valid path next to it is changed as asked.
#[tokio::test]
async fn should_skip_a_path_out_of_the_content_area_when_changing_a_list() {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-change-refused").unwrap();
    create_instance_dir(&location_info, instance_id).await;
    let instance_dir = location_info.instance_dir(instance_id);

    // Targets each path would reach if it were joined as it is.
    std::fs::write(temp_dir.path().join("escaped.jar"), b"outside").unwrap();
    std::fs::write(temp_dir.path().join("escaped.jar.disabled"), b"outside").unwrap();
    std::fs::create_dir_all(instance_dir.join(".metadata")).unwrap();
    std::fs::write(instance_dir.join(".metadata/instance.json"), b"{}").unwrap();
    let absolute = temp_dir.path().join("escaped.jar").display().to_string();

    let valid = instance_dir.join("mods/test.jar");
    for path in [
        "../../escaped.jar",
        "mods/../../../escaped.jar",
        absolute.as_str(),
        "/escaped.jar",
        ".metadata/instance.json",
        ".Metadata/instance.json",
    ] {
        let paths = ["mods/test.jar".to_owned(), path.to_owned()];
        std::fs::create_dir_all(valid.parent().unwrap()).unwrap();
        std::fs::write(&valid, b"jar").unwrap();

        service
            .disable_content_files(instance_id, &paths)
            .await
            .expect(path);
        assert!(
            instance_dir.join("mods/test.jar.disabled").exists(),
            "{path}"
        );
        service
            .enable_content_files(instance_id, &paths)
            .await
            .expect(path);
        assert!(valid.exists(), "{path}");
        service
            .remove_content_files(instance_id, &paths)
            .await
            .expect(path);
        assert!(!valid.exists(), "{path}");
    }

    assert!(temp_dir.path().join("escaped.jar").exists());
    assert!(temp_dir.path().join("escaped.jar.disabled").exists());
    assert!(instance_dir.join(".metadata/instance.json").exists());
    assert!(
        !instance_dir
            .join(".metadata/instance.json.disabled")
            .exists()
    );
}

/// A plain path whose folder is a symlink out of the instance is refused.
#[tokio::test]
async fn should_refuse_to_change_a_file_when_a_symlink_leads_out_of_the_instance() {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = &InstanceId::try_from("test-change-symlink").unwrap();
    let instance_dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(&instance_dir).await.unwrap();
    let outside = temp_dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("escaped.jar"), b"outside").unwrap();

    link_dir(&outside, &instance_dir.join("mods"));

    let paths = ["mods/escaped.jar".to_owned()];
    for (operation, result) in change_all(&service, instance_id, &paths).await {
        let err = result.expect_err(operation);
        assert!(
            matches!(err, InstanceError::PathEscapesInstance { .. }),
            "{operation}: {err:?}"
        );
    }

    assert!(outside.join("escaped.jar").exists());
    assert!(!outside.join("escaped.jar.disabled").exists());
}
