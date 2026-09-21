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

// ── write_instance_file / read_instance_file (T-1.3) ──

use crate::features::instance::domain::InstanceError;
use crate::features::instance::infra::MAX_INSTANCE_FILE_BYTES;

/// The service assumes the instance directory is already there — `require_instance` in the host
/// function is what guarantees the instance exists at all.
async fn setup_instance(instance_id: &str) -> (TempDir, Arc<LocationInfo>, FsInstanceFileService) {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(location_info.clone());
    service
        .create_instance_dir(instance_id)
        .await
        .expect("create_instance_dir should succeed");
    (temp_dir, location_info, service)
}

#[tokio::test]
async fn test_write_instance_file_creates_the_file_inside_the_instance() {
    let instance_id = "write-plain";
    let (_temp_dir, location_info, service) = setup_instance(instance_id).await;

    service
        .write_instance_file(instance_id, "mods/example.jar", b"jar bytes")
        .await
        .expect("write_instance_file should succeed");

    let written = location_info
        .instance_dir(instance_id)
        .join("mods")
        .join("example.jar");
    assert!(written.exists(), "file should exist inside the instance dir");
    assert_eq!(tokio::fs::read(&written).await.unwrap(), b"jar bytes");
}

#[tokio::test]
async fn test_write_instance_file_replaces_existing_content() {
    let instance_id = "write-replace";
    let (_temp_dir, location_info, service) = setup_instance(instance_id).await;

    service
        .write_instance_file(instance_id, "pack.toml", b"first")
        .await
        .unwrap();
    service
        .write_instance_file(instance_id, "pack.toml", b"second")
        .await
        .unwrap();

    let written = location_info.instance_dir(instance_id).join("pack.toml");
    assert_eq!(tokio::fs::read(&written).await.unwrap(), b"second");
}

#[tokio::test]
async fn test_read_instance_file_returns_what_was_written() {
    let instance_id = "read-back";
    let (_temp_dir, _location_info, service) = setup_instance(instance_id).await;

    service
        .write_instance_file(instance_id, "config/a.json", b"{}")
        .await
        .unwrap();

    let read = service
        .read_instance_file(instance_id, "config/a.json")
        .await
        .expect("read_instance_file should succeed");
    assert_eq!(read, b"{}");
}

#[tokio::test]
async fn test_write_instance_file_rejects_parent_traversal() {
    let instance_id = "write-traversal";
    let (temp_dir, _location_info, service) = setup_instance(instance_id).await;

    let err = service
        .write_instance_file(instance_id, "../escaped.txt", b"pwned")
        .await
        .expect_err("`..` must be rejected");
    assert!(matches!(err, InstanceError::InvalidRelativePath { .. }), "{err}");

    assert!(
        !temp_dir.path().join("instances").join("escaped.txt").exists(),
        "nothing may be written outside the instance"
    );
}

#[tokio::test]
async fn test_write_instance_file_rejects_absolute_path() {
    let instance_id = "write-absolute";
    let (_temp_dir, _location_info, service) = setup_instance(instance_id).await;

    for path in ["/etc/passwd", "C:/Windows/evil.dll", "/instances/x/a.jar"] {
        let err = service
            .write_instance_file(instance_id, path, b"pwned")
            .await
            .expect_err("an absolute path must be rejected");
        assert!(
            matches!(err, InstanceError::InvalidRelativePath { .. }),
            "`{path}`: {err}"
        );
    }
}

#[tokio::test]
async fn test_write_instance_file_rejects_symlink_out_of_the_instance() {
    let instance_id = "write-symlink";
    let (temp_dir, location_info, service) = setup_instance(instance_id).await;

    let outside = temp_dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();

    let link = location_info.instance_dir(instance_id).join("mods");
    let linked = {
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(&outside, &link).is_ok()
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside, &link).is_ok()
        }
    };
    if !linked {
        eprintln!("skipped: this host does not allow creating directory symlinks");
        return;
    }

    let err = service
        .write_instance_file(instance_id, "mods/escaped.jar", b"pwned")
        .await
        .expect_err("a write through a symlink out of the instance must be rejected");
    assert!(matches!(err, InstanceError::PathEscapesInstance { .. }), "{err}");

    assert!(
        !outside.join("escaped.jar").exists(),
        "nothing may be written through the symlink"
    );
}

#[tokio::test]
async fn test_write_instance_file_rejects_oversized_payload() {
    let instance_id = "write-too-large";
    let (_temp_dir, location_info, service) = setup_instance(instance_id).await;

    // One byte over the cap is enough; the check happens before anything is written.
    let oversized = vec![0_u8; usize::try_from(MAX_INSTANCE_FILE_BYTES).unwrap() + 1];

    let err = service
        .write_instance_file(instance_id, "big.bin", &oversized)
        .await
        .expect_err("an oversized payload must be rejected");
    assert!(matches!(err, InstanceError::FileTooLarge { .. }), "{err}");

    assert!(
        !location_info.instance_dir(instance_id).join("big.bin").exists(),
        "an oversized write must leave no partial file"
    );
}

#[tokio::test]
async fn test_read_instance_file_rejects_a_directory() {
    let instance_id = "read-dir";
    let (_temp_dir, _location_info, service) = setup_instance(instance_id).await;

    service
        .write_instance_file(instance_id, "mods/a.jar", b"x")
        .await
        .unwrap();

    let err = service
        .read_instance_file(instance_id, "mods")
        .await
        .expect_err("a directory is not a readable file");
    assert!(matches!(err, InstanceError::InvalidRelativePath { .. }), "{err}");
}

#[tokio::test]
async fn test_read_instance_file_fails_for_a_missing_file() {
    let instance_id = "read-missing";
    let (_temp_dir, _location_info, service) = setup_instance(instance_id).await;

    assert!(
        service
            .read_instance_file(instance_id, "nope.txt")
            .await
            .is_err()
    );
}
