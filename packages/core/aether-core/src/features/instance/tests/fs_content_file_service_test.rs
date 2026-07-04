use std::sync::Arc;

use tempfile::TempDir;

use crate::features::instance::{app::ContentFileService, infra::FsContentFileService};
use crate::features::settings::LocationInfo;

fn setup_temp_location() -> (TempDir, Arc<LocationInfo>) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let location_info = Arc::new(LocationInfo::new(
        temp_dir.path().join("settings"),
        temp_dir.path().to_path_buf(),
    ));
    (temp_dir, location_info)
}

async fn create_instance_dir(location_info: &LocationInfo, instance_id: &str) {
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
async fn test_fs_content_service_removes_single_file() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-instance-1";

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
async fn test_fs_content_service_removes_multiple_files() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-instance-2";

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
async fn test_fs_content_service_removes_disabled_file() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-instance-3";

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
async fn test_fs_content_service_nonexistent_file_is_idempotent() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-instance-4";

    // No files created at all — should not fail
    service
        .remove_content_files(instance_id, &["mods/nonexistent.jar".to_string()])
        .await
        .expect("remove_content_files should succeed for non-existent files");
}

// ─── Enable / Disable tests ────────────────────────────────────

#[tokio::test]
async fn test_fs_content_service_enables_file() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-enable-1";

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
async fn test_fs_content_service_disables_file() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-disable-1";

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
async fn test_fs_content_service_enable_already_enabled_is_idempotent() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-enable-idempotent";

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
async fn test_fs_content_service_disable_already_disabled_is_idempotent() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-disable-idempotent";

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
async fn test_fs_content_service_enable_multiple_files() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-enable-multi";

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
async fn test_fs_content_service_disable_multiple_files() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-disable-multi";

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
async fn test_fs_content_service_nonexistent_file_enable_disable_is_idempotent() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsContentFileService::new(location_info.clone());
    let instance_id = "test-nonexistent-toggle";

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
