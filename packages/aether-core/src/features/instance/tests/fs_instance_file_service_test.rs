use std::sync::Arc;

use tempfile::TempDir;

use crate::features::instance::InstanceId;
use crate::features::instance::app::InstanceFileService;
use crate::features::instance::infra::FsInstanceFileService;
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
async fn should_remove_the_instance_dir_when_it_exists() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    );
    let instance_id = &InstanceId::try_from("test-instance-123").unwrap();

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
async fn should_remove_nested_files_when_the_instance_dir_is_removed() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    );
    let instance_id = &InstanceId::try_from("test-instance-nested").unwrap();

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
async fn should_succeed_when_the_instance_dir_does_not_exist() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    );
    let instance_id = &InstanceId::try_from("nonexistent-instance").unwrap();

    let instance_dir = location_info.instance_dir(instance_id);
    assert!(!instance_dir.exists(), "Dir should not exist");

    // Act — should not fail
    service
        .remove_instance_dir(instance_id)
        .await
        .expect("remove_instance_dir should succeed for non-existent dir");
}

#[tokio::test]
async fn should_leave_nothing_behind_and_sweep_old_leftovers_when_the_instance_dir_is_removed() {
    let (_temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    );
    let instance_id = &InstanceId::try_from("to-remove").unwrap();
    create_instance_dir(&location_info, instance_id).await;
    let leftover = location_info.instances_dir().join(".removing-old-1234");
    tokio::fs::create_dir_all(leftover.join("mods"))
        .await
        .unwrap();

    service.remove_instance_dir(instance_id).await.unwrap();

    let mut names = Vec::new();
    let mut entries = tokio::fs::read_dir(location_info.instances_dir())
        .await
        .unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        names.push(entry.file_name());
    }
    assert!(names.is_empty(), "left behind: {names:?}");
}

// ── write_instance_file / read_instance_file ──

use crate::features::instance::domain::InstanceError;

/// The service assumes the instance directory is already there — `require_instance` in the host
/// function is what guarantees the instance exists at all.
async fn setup_instance(
    instance_id: &InstanceId,
) -> (TempDir, Arc<LocationInfo>, FsInstanceFileService) {
    let (temp_dir, location_info) = setup_temp_location();
    let service = FsInstanceFileService::new(
        location_info.clone(),
        crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
    );
    service
        .create_instance_dir(instance_id)
        .await
        .expect("create_instance_dir should succeed");
    (temp_dir, location_info, service)
}

#[tokio::test]
async fn should_create_the_file_inside_the_instance_when_writing() {
    let instance_id = &InstanceId::try_from("write-plain").unwrap();
    let (_temp_dir, location_info, service) = setup_instance(instance_id).await;

    service
        .write_instance_file(instance_id, "mods/example.jar", b"jar bytes")
        .await
        .expect("write_instance_file should succeed");

    let written = location_info
        .instance_dir(instance_id)
        .join("mods")
        .join("example.jar");
    assert!(
        written.exists(),
        "file should exist inside the instance dir"
    );
    assert_eq!(tokio::fs::read(&written).await.unwrap(), b"jar bytes");
}

#[tokio::test]
async fn should_replace_existing_content_when_the_file_is_written_again() {
    let instance_id = &InstanceId::try_from("write-replace").unwrap();
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
async fn should_return_what_was_written_when_the_file_is_read() {
    let instance_id = &InstanceId::try_from("read-back").unwrap();
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
async fn should_reject_a_write_when_the_path_traverses_to_the_parent() {
    let instance_id = &InstanceId::try_from("write-traversal").unwrap();
    let (temp_dir, _location_info, service) = setup_instance(instance_id).await;

    let err = service
        .write_instance_file(instance_id, "../escaped.txt", b"pwned")
        .await
        .expect_err("`..` must be rejected");
    assert!(
        matches!(err, InstanceError::InvalidRelativePath { .. }),
        "{err}"
    );

    assert!(
        !temp_dir
            .path()
            .join("instances")
            .join("escaped.txt")
            .exists(),
        "nothing may be written outside the instance"
    );
}

#[tokio::test]
async fn should_reject_a_write_when_the_path_is_absolute() {
    let instance_id = &InstanceId::try_from("write-absolute").unwrap();
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
async fn should_reject_a_write_when_a_symlink_leads_out_of_the_instance() {
    let instance_id = &InstanceId::try_from("write-symlink").unwrap();
    let (temp_dir, location_info, service) = setup_instance(instance_id).await;

    let outside = temp_dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();

    let link = location_info.instance_dir(instance_id).join("mods");
    link_dir(&outside, &link);

    let err = service
        .write_instance_file(instance_id, "mods/escaped.jar", b"pwned")
        .await
        .expect_err("a write through a symlink out of the instance must be rejected");
    assert!(
        matches!(err, InstanceError::PathEscapesInstance { .. }),
        "{err}"
    );

    assert!(
        !outside.join("escaped.jar").exists(),
        "nothing may be written through the symlink"
    );
}

#[tokio::test]
async fn should_reject_a_write_when_the_payload_is_oversized() {
    let instance_id = &InstanceId::try_from("write-too-large").unwrap();
    let (_temp_dir, location_info, service) = setup_instance(instance_id).await;

    // One byte over the cap is enough; the check happens before anything is written.
    let oversized = vec![
        0_u8;
        usize::try_from(
            crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes
        )
        .unwrap()
            + 1
    ];

    let err = service
        .write_instance_file(instance_id, "big.bin", &oversized)
        .await
        .expect_err("an oversized payload must be rejected");
    assert!(matches!(err, InstanceError::FileTooLarge { .. }), "{err}");

    assert!(
        !location_info
            .instance_dir(instance_id)
            .join("big.bin")
            .exists(),
        "an oversized write must leave no partial file"
    );
}

#[tokio::test]
async fn should_reject_a_read_when_the_path_is_a_directory() {
    let instance_id = &InstanceId::try_from("read-dir").unwrap();
    let (_temp_dir, _location_info, service) = setup_instance(instance_id).await;

    service
        .write_instance_file(instance_id, "mods/a.jar", b"x")
        .await
        .unwrap();

    let err = service
        .read_instance_file(instance_id, "mods")
        .await
        .expect_err("a directory is not a readable file");
    assert!(
        matches!(err, InstanceError::InvalidRelativePath { .. }),
        "{err}"
    );
}

#[tokio::test]
async fn should_fail_a_read_when_the_file_is_missing() {
    let instance_id = &InstanceId::try_from("read-missing").unwrap();
    let (_temp_dir, _location_info, service) = setup_instance(instance_id).await;

    assert!(
        service
            .read_instance_file(instance_id, "nope.txt")
            .await
            .is_err()
    );
}
