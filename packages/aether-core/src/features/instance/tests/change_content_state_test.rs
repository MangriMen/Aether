use std::sync::Arc;

use async_trait::async_trait;
use rstest::rstest;
use tempfile::TempDir;

use crate::features::events::{Event, EventEmitter, EventError, SharedEventEmitter};
use crate::features::instance::app::ContentFileService;
use crate::features::instance::infra::FsContentFileService;
use crate::features::instance::{
    ChangeContentState, ChangeContentStateUseCase, ContentStateAction, InstanceError, InstanceId,
    InstanceLocks,
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
    get_use_case_with_locks(content_file_service, Arc::new(InstanceLocks::new()))
}

fn get_use_case_with_locks(
    content_file_service: Arc<dyn ContentFileService>,
    locks: Arc<InstanceLocks>,
) -> ChangeContentStateUseCase {
    let event_emitter: SharedEventEmitter = Arc::new(MockEventEmitter);
    ChangeContentStateUseCase::new(event_emitter, content_file_service, locks)
}

// ─── Tests ─────────────────────────────────────────────────────

/// Creates `mods/` under the instance directory and the given files in it.
async fn put_files(dir: &std::path::Path, files: &[&str]) {
    tokio::fs::create_dir_all(dir.join("mods"))
        .await
        .expect("Failed to create dirs");
    for file in files {
        tokio::fs::write(dir.join(file), b"data").await.unwrap();
    }
}

#[rstest]
#[case::enable(
    &["mods/test.jar.disabled"],
    ContentStateAction::Enable,
    &["mods/test.jar"],
    &["mods/test.jar.disabled"],
)]
#[case::disable(
    &["mods/test.jar"],
    ContentStateAction::Disable,
    &["mods/test.jar.disabled"],
    &["mods/test.jar"],
)]
#[case::already_enabled(
    &["mods/test.jar"],
    ContentStateAction::Enable,
    &["mods/test.jar"],
    &[],
)]
#[case::already_disabled(
    &["mods/test.jar.disabled"],
    ContentStateAction::Disable,
    &["mods/test.jar.disabled"],
    &[],
)]
#[tokio::test]
async fn should_change_a_file_state_when_one_file_is_given(
    #[case] initial: &[&str],
    #[case] action: ContentStateAction,
    #[case] present: &[&str],
    #[case] absent: &[&str],
) {
    let (_temp, location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = InstanceId::try_from("test-change-state-usecase").unwrap();
    let dir = location_info.instance_dir(&instance_id);
    put_files(&dir, initial).await;

    use_case
        .execute(ChangeContentState::single(
            instance_id.clone(),
            "mods/test.jar".to_string(),
            action,
        ))
        .await
        .expect("Changing the state should succeed");

    for file in present {
        assert!(dir.join(file).exists(), "{file} should exist");
    }
    for file in absent {
        assert!(!dir.join(file).exists(), "{file} should not exist");
    }
}

#[rstest]
#[case::enable(
    &["mods/a.jar.disabled", "mods/b.jar.disabled"],
    ContentStateAction::Enable,
    &["mods/a.jar", "mods/b.jar"],
)]
#[case::disable(
    &["mods/a.jar", "mods/b.jar"],
    ContentStateAction::Disable,
    &["mods/a.jar.disabled", "mods/b.jar.disabled"],
)]
#[tokio::test]
async fn should_change_file_states_when_several_are_given(
    #[case] initial: &[&str],
    #[case] action: ContentStateAction,
    #[case] present: &[&str],
) {
    let (_temp, location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = InstanceId::try_from("test-change-state-multi-usecase").unwrap();
    let dir = location_info.instance_dir(&instance_id);
    put_files(&dir, initial).await;

    use_case
        .execute(ChangeContentState::multiple(
            instance_id.clone(),
            vec!["mods/a.jar".to_string(), "mods/b.jar".to_string()],
            action,
        ))
        .await
        .expect("Changing several states should succeed");

    for file in present {
        assert!(dir.join(file).exists(), "{file} should exist");
    }
}

#[tokio::test]
async fn should_succeed_when_the_file_does_not_exist() {
    let (_temp, _location_info, service) = setup();
    let use_case = get_use_case(service);
    let instance_id = InstanceId::try_from("test-nonexistent").unwrap();

    use_case
        .execute(ChangeContentState::single(
            instance_id.clone(),
            "mods/nonexistent.jar".to_string(),
            ContentStateAction::Enable,
        ))
        .await
        .expect("Enable on nonexistent should not fail");

    use_case
        .execute(ChangeContentState::single(
            instance_id.clone(),
            "mods/nonexistent.jar".to_string(),
            ContentStateAction::Disable,
        ))
        .await
        .expect("Disable on nonexistent should not fail");
}

async fn put_enabled_file(location_info: &LocationInfo, instance_id: &str) -> std::path::PathBuf {
    let dir = location_info.instance_dir(&InstanceId::try_from(instance_id).unwrap());
    tokio::fs::create_dir_all(dir.join("mods")).await.unwrap();
    tokio::fs::write(dir.join("mods/test.jar"), b"data")
        .await
        .unwrap();
    dir
}

#[tokio::test]
async fn should_refuse_the_change_when_a_plan_holds_the_lock_past_the_wait() {
    let (_temp, location_info, service) = setup();
    let locks = Arc::new(InstanceLocks::new());
    let use_case = get_use_case_with_locks(service, locks.clone())
        .with_lock_wait(std::time::Duration::from_millis(20));
    let dir = put_enabled_file(&location_info, "busy").await;
    let _held = locks.lock(&InstanceId::try_from("busy").unwrap()).await;

    let result = use_case
        .execute(ChangeContentState::single(
            InstanceId::try_from("busy").unwrap(),
            "mods/test.jar".to_string(),
            ContentStateAction::Disable,
        ))
        .await;

    assert!(matches!(
        result,
        Err(InstanceError::InstanceStillInstalling { .. })
    ));
    assert!(dir.join("mods/test.jar").exists());
}

#[tokio::test]
async fn should_change_the_state_when_the_lock_is_released_within_the_wait() {
    let (_temp, location_info, service) = setup();
    let locks = Arc::new(InstanceLocks::new());
    let use_case = get_use_case_with_locks(service, locks.clone());
    let dir = put_enabled_file(&location_info, "brief").await;
    let held = locks.lock(&InstanceId::try_from("brief").unwrap()).await;

    let mut change = std::pin::pin!(use_case.execute(ChangeContentState::single(
        InstanceId::try_from("brief").unwrap(),
        "mods/test.jar".to_string(),
        ContentStateAction::Disable,
    )));
    assert!(futures::poll!(&mut change).is_pending());
    assert!(dir.join("mods/test.jar").exists());

    drop(held);
    change.await.unwrap();
    assert!(dir.join("mods/test.jar.disabled").exists());
}
