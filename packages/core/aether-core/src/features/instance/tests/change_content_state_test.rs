use std::sync::Arc;

use async_trait::async_trait;
use tempfile::TempDir;

use crate::features::events::{Event, EventEmitter, EventError, SharedEventEmitter};
use crate::features::instance::app::ContentFileService;
use crate::features::instance::infra::FsContentFileService;
use crate::features::instance::{
    ChangeContentState, ChangeContentStateUseCase, ContentStateAction,
};
use crate::features::settings::LocationInfo;

// ─── Mock event emitter ────────────────────────────────────────

struct MockEventEmitter;

#[async_trait]
impl EventEmitter<Event> for MockEventEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }

    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {
        // no-op
    }
}

// ─── Test infrastructure ───────────────────────────────────────

fn setup() -> (TempDir, Arc<LocationInfo>, Arc<FsContentFileService>) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let location_info = Arc::new(LocationInfo::new(
        temp_dir.path().join("settings"),
        temp_dir.path().to_path_buf(),
    ));
    let service = FsContentFileService::new(location_info.clone());
    (temp_dir, location_info, Arc::new(service))
}

fn get_use_case(content_file_service: Arc<dyn ContentFileService>) -> ChangeContentStateUseCase {
    let event_emitter: SharedEventEmitter = Arc::new(MockEventEmitter);
    ChangeContentStateUseCase::new(event_emitter, content_file_service)
}

// ─── Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_change_content_state_enable_single_file() {
    let (_temp, location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = "test-enable-usecase";

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");
    tokio::fs::write(dir.join("mods/test.jar.disabled"), b"data")
        .await
        .unwrap();

    use_case
        .execute(ChangeContentState::single(
            instance_id.to_string(),
            "mods/test.jar".to_string(),
            ContentStateAction::Enable,
        ))
        .await
        .expect("Enable should succeed");

    assert!(dir.join("mods/test.jar").exists(), "File should be enabled");
    assert!(
        !dir.join("mods/test.jar.disabled").exists(),
        "Disabled file should not exist"
    );
}

#[tokio::test]
async fn test_change_content_state_disable_single_file() {
    let (_temp, location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = "test-disable-usecase";

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");
    tokio::fs::write(dir.join("mods/test.jar"), b"data")
        .await
        .unwrap();

    use_case
        .execute(ChangeContentState::single(
            instance_id.to_string(),
            "mods/test.jar".to_string(),
            ContentStateAction::Disable,
        ))
        .await
        .expect("Disable should succeed");

    assert!(
        dir.join("mods/test.jar.disabled").exists(),
        "File should be disabled"
    );
    assert!(
        !dir.join("mods/test.jar").exists(),
        "Original file should not exist"
    );
}

#[tokio::test]
async fn test_change_content_state_enable_multiple_files() {
    let (_temp, location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = "test-enable-multi-usecase";

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

    use_case
        .execute(ChangeContentState::multiple(
            instance_id.to_string(),
            vec!["mods/a.jar".to_string(), "mods/b.jar".to_string()],
            ContentStateAction::Enable,
        ))
        .await
        .expect("Enable multiple should succeed");

    assert!(dir.join("mods/a.jar").exists(), "a.jar should be enabled");
    assert!(dir.join("mods/b.jar").exists(), "b.jar should be enabled");
}

#[tokio::test]
async fn test_change_content_state_disable_multiple_files() {
    let (_temp, location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = "test-disable-multi-usecase";

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

    use_case
        .execute(ChangeContentState::multiple(
            instance_id.to_string(),
            vec!["mods/a.jar".to_string(), "mods/b.jar".to_string()],
            ContentStateAction::Disable,
        ))
        .await
        .expect("Disable multiple should succeed");

    assert!(
        dir.join("mods/a.jar.disabled").exists(),
        "a.jar should be disabled"
    );
    assert!(
        dir.join("mods/b.jar.disabled").exists(),
        "b.jar should be disabled"
    );
}

#[tokio::test]
async fn test_change_content_state_already_enabled_is_idempotent() {
    let (_temp, location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = "test-already-enabled";

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");
    tokio::fs::write(dir.join("mods/test.jar"), b"data")
        .await
        .unwrap();

    use_case
        .execute(ChangeContentState::single(
            instance_id.to_string(),
            "mods/test.jar".to_string(),
            ContentStateAction::Enable,
        ))
        .await
        .expect("Enable should succeed even if already enabled");

    assert!(
        dir.join("mods/test.jar").exists(),
        "File should still exist"
    );
}

#[tokio::test]
async fn test_change_content_state_already_disabled_is_idempotent() {
    let (_temp, location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = "test-already-disabled";

    let dir = location_info.instance_dir(instance_id);
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");
    tokio::fs::write(dir.join("mods/test.jar.disabled"), b"data")
        .await
        .unwrap();

    use_case
        .execute(ChangeContentState::single(
            instance_id.to_string(),
            "mods/test.jar".to_string(),
            ContentStateAction::Disable,
        ))
        .await
        .expect("Disable should succeed even if already disabled");

    assert!(
        dir.join("mods/test.jar.disabled").exists(),
        "Disabled file should still exist"
    );
}

#[tokio::test]
async fn test_change_content_state_nonexistent_file_is_idempotent() {
    let (_temp, _location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = "test-nonexistent";

    use_case
        .execute(ChangeContentState::single(
            instance_id.to_string(),
            "mods/nonexistent.jar".to_string(),
            ContentStateAction::Enable,
        ))
        .await
        .expect("Enable on nonexistent should not fail");

    use_case
        .execute(ChangeContentState::single(
            instance_id.to_string(),
            "mods/nonexistent.jar".to_string(),
            ContentStateAction::Disable,
        ))
        .await
        .expect("Disable on nonexistent should not fail");
}
