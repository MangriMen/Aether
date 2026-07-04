use std::sync::Arc;

use tempfile::TempDir;

use crate::features::instance::app::InstanceFileService;
use crate::features::instance::infra::FsInstanceFileService;
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
    let path = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(&path)
        .await
        .expect("Failed to create instance dir");

    // Create some nested files inside to simulate a real instance
    let sub_dir = path.join("mods");
    tokio::fs::create_dir_all(&sub_dir)
        .await
        .expect("Failed to create sub dir");

    tokio::fs::write(sub_dir.join("test.jar"), b"fake content")
        .await
        .expect("Failed to write test file");

    tokio::fs::write(path.join("launch.cfg"), b"config")
        .await
        .expect("Failed to write config file");
}

#[tokio::test]
async fn test_fs_service_removes_instance_dir() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(location_info.clone());
    let instance_id = "test-instance-123";

    // Create a fake instance directory
    create_instance_dir(&location_info, instance_id).await;
    let instance_dir = location_info.instance_dir(instance_id);
    assert!(
        instance_dir.exists(),
        "Instance dir should exist before removal"
    );

    // Act
    service
        .remove_instance_dir(instance_id)
        .await
        .expect("remove_instance_dir should succeed");

    // Assert
    assert!(!instance_dir.exists(), "Instance dir should be removed");
}

#[tokio::test]
async fn test_fs_service_removes_nested_files() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(location_info.clone());
    let instance_id = "test-instance-nested";

    create_instance_dir(&location_info, instance_id).await;
    let instance_dir = location_info.instance_dir(instance_id);

    // Verify nested structure exists
    assert!(instance_dir.join("mods").join("test.jar").exists());
    assert!(instance_dir.join("launch.cfg").exists());

    // Act
    service
        .remove_instance_dir(instance_id)
        .await
        .expect("remove_instance_dir should succeed");

    // Assert — everything is gone
    assert!(
        !instance_dir.exists(),
        "Instance dir should be completely removed"
    );
}

#[tokio::test]
async fn test_fs_service_nonexistent_dir_is_idempotent() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(location_info.clone());
    let instance_id = "nonexistent-instance";

    let instance_dir = location_info.instance_dir(instance_id);
    assert!(!instance_dir.exists(), "Dir should not exist");

    // Act — should not fail
    service
        .remove_instance_dir(instance_id)
        .await
        .expect("remove_instance_dir should succeed for non-existent dir");
}
