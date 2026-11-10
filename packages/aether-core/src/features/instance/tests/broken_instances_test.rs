use std::sync::Arc;

use async_trait::async_trait;
use tempfile::TempDir;

use crate::features::{
    events::test_support::ChannelEmitter,
    instance::{
        BrokenInstanceOrigin, BrokenInstanceReason, Instance, InstanceBuilder, InstanceError,
        InstanceId, InstanceName, InstanceStorage, ListBrokenInstancesUseCase,
        ListBrokenInstancesUseCasePort, RemoveBrokenInstanceUseCase,
        RemoveBrokenInstanceUseCasePort, RemoveInstanceUseCasePort,
        infra::{
            EventEmittingInstanceStorage, FsInstanceStorage, LegacyInstanceScanner,
            SqliteInstanceStorage,
        },
    },
    minecraft::{GameVersion, ModLoader},
    settings::LocationInfo,
};

struct Setup {
    _temp: TempDir,
    location_info: Arc<LocationInfo>,
    sqlite: Arc<SqliteInstanceStorage>,
    list: Arc<ListBrokenInstancesUseCase>,
}

async fn setup() -> Setup {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let sqlite = Arc::new(SqliteInstanceStorage::in_memory().await);
    let list = Arc::new(ListBrokenInstancesUseCase::new(
        sqlite.clone(),
        sqlite.clone(),
        Arc::new(LegacyInstanceScanner::new(location_info.clone())),
        location_info.clone(),
    ));
    Setup {
        _temp: temp,
        location_info,
        sqlite,
        list,
    }
}

fn instance(id: &str) -> Instance {
    InstanceBuilder::new(
        InstanceId::try_from(id).unwrap(),
        InstanceName::parse(id).unwrap(),
        GameVersion::from_static("1.21.1"),
        ModLoader::Vanilla,
    )
    .build()
}

/// A database row of `id` whose game version cannot be read.
async fn broken_row(setup: &Setup, id: &str) {
    setup.sqlite.upsert(&instance(id)).await.unwrap();
    sqlx::query("UPDATE instances SET game_version = '../x' WHERE id = ?")
        .bind(id)
        .execute(setup.sqlite.pool())
        .await
        .unwrap();
}

fn legacy_file(setup: &Setup, id: &str) -> std::path::PathBuf {
    setup
        .location_info
        .instance_metadata_file(&InstanceId::try_from(id).unwrap())
}

fn write_legacy(setup: &Setup, id: &str, content: &[u8]) {
    let file = legacy_file(setup, id);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, content).unwrap();
}

/// A legacy `instance.json` of a readable instance.
async fn write_readable_legacy(setup: &Setup, id: &str) {
    FsInstanceStorage::new(setup.location_info.clone())
        .upsert(&instance(id))
        .await
        .unwrap();
}

async fn listed(setup: &Setup) -> Vec<(String, BrokenInstanceOrigin, BrokenInstanceReason)> {
    setup
        .list
        .execute()
        .await
        .unwrap()
        .into_iter()
        .map(|broken| (broken.id, broken.origin, broken.reason))
        .collect()
}

#[tokio::test]
async fn should_list_a_broken_row_with_its_folder_when_the_record_cannot_be_read() {
    let setup = setup().await;
    broken_row(&setup, "one").await;

    let found = setup.list.execute().await.unwrap();

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].origin, BrokenInstanceOrigin::Database);
    assert_eq!(
        found[0].dir.as_deref(),
        Some(
            setup
                .location_info
                .instance_dir(&InstanceId::try_from("one").unwrap())
                .as_path()
        )
    );
}

#[tokio::test]
async fn should_list_no_folder_when_the_id_of_a_broken_row_names_none() {
    let setup = setup().await;
    setup.sqlite.upsert(&instance("a")).await.unwrap();
    sqlx::query("UPDATE instances SET id = '../x'")
        .execute(setup.sqlite.pool())
        .await
        .unwrap();

    let found = setup.list.execute().await.unwrap();

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "../x");
    assert!(found[0].dir.is_none());
}

#[tokio::test]
async fn should_list_a_legacy_folder_when_its_file_cannot_be_read() {
    let setup = setup().await;
    write_legacy(&setup, "old", b"{ not json");

    let found = listed(&setup).await;

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, "old");
    assert_eq!(found[0].1, BrokenInstanceOrigin::LegacyFile);
    assert!(matches!(
        found[0].2,
        BrokenInstanceReason::UnreadableFile { .. }
    ));
}

#[tokio::test]
async fn should_list_a_legacy_folder_as_not_migrated_when_its_file_reads_but_has_no_record() {
    let setup = setup().await;
    write_readable_legacy(&setup, "old").await;

    assert_eq!(
        listed(&setup).await,
        [(
            "old".to_owned(),
            BrokenInstanceOrigin::LegacyFile,
            BrokenInstanceReason::NotMigrated
        )]
    );
}

#[tokio::test]
async fn should_not_list_a_folder_when_it_has_no_legacy_file() {
    let setup = setup().await;
    // An instance made since the move into the database, or being created now.
    std::fs::create_dir_all(
        setup
            .location_info
            .instance_dir(&InstanceId::try_from("new").unwrap()),
    )
    .unwrap();

    assert!(listed(&setup).await.is_empty());
}

#[tokio::test]
async fn should_not_list_a_folder_when_its_legacy_file_was_set_aside() {
    let setup = setup().await;
    write_legacy(&setup, "done", b"{}");
    let file = legacy_file(&setup, "done");
    std::fs::rename(&file, file.with_extension("json.migrated")).unwrap();

    assert!(listed(&setup).await.is_empty());
}

#[tokio::test]
async fn should_not_list_a_folder_when_the_database_has_a_record_of_it() {
    let setup = setup().await;
    setup.sqlite.upsert(&instance("live")).await.unwrap();
    // A file left in the folder of a working instance is not the instance to delete.
    write_legacy(&setup, "live", b"{ not json");

    assert!(listed(&setup).await.is_empty());
}

#[tokio::test]
async fn should_leave_the_legacy_file_as_it_was_when_it_is_only_looked_at() {
    let setup = setup().await;
    write_readable_legacy(&setup, "old").await;
    let before = std::fs::read(legacy_file(&setup, "old")).unwrap();

    let _ = setup.list.execute().await.unwrap();

    assert_eq!(std::fs::read(legacy_file(&setup, "old")).unwrap(), before);
}

#[derive(Default)]
struct RecordingRemove(std::sync::Mutex<Vec<String>>);

#[async_trait]
impl RemoveInstanceUseCasePort for RecordingRemove {
    async fn execute(&self, instance_id: InstanceId) -> Result<(), InstanceError> {
        self.0.lock().unwrap().push(instance_id.to_string());
        Ok(())
    }
}

fn remover(setup: &Setup) -> (RemoveBrokenInstanceUseCase, Arc<RecordingRemove>) {
    let recording = Arc::new(RecordingRemove::default());
    (
        RemoveBrokenInstanceUseCase::new(
            setup.sqlite.clone(),
            setup.list.clone() as Arc<dyn ListBrokenInstancesUseCasePort>,
            setup.sqlite.clone(),
            recording.clone(),
        ),
        recording,
    )
}

#[tokio::test]
async fn should_remove_through_the_instance_removal_when_the_instance_is_broken() {
    let setup = setup().await;
    broken_row(&setup, "one").await;
    let (remove, recording) = remover(&setup);

    remove.execute("one".to_owned()).await.unwrap();

    assert_eq!(*recording.0.lock().unwrap(), ["one"]);
}

#[tokio::test]
async fn should_refuse_to_remove_when_the_instance_is_not_broken() {
    let setup = setup().await;
    setup.sqlite.upsert(&instance("fine")).await.unwrap();
    let (remove, recording) = remover(&setup);

    let result = remove.execute("fine".to_owned()).await;

    assert!(
        matches!(result, Err(InstanceError::NotBroken { .. })),
        "{result:?}"
    );
    assert!(recording.0.lock().unwrap().is_empty());
    assert!(
        setup
            .sqlite
            .get(&InstanceId::try_from("fine").unwrap())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn should_delete_only_the_record_when_the_id_names_no_folder() {
    let setup = setup().await;
    setup.sqlite.upsert(&instance("a")).await.unwrap();
    sqlx::query("UPDATE instances SET id = '../x'")
        .execute(setup.sqlite.pool())
        .await
        .unwrap();
    // What the id would point at, were it taken for a path.
    let outside = setup.location_info.instances_dir().join("../x");
    std::fs::create_dir_all(&outside).unwrap();
    let (remove, recording) = remover(&setup);

    remove.execute("../x".to_owned()).await.unwrap();

    assert!(
        recording.0.lock().unwrap().is_empty(),
        "no folder was involved"
    );
    assert!(setup.list.execute().await.unwrap().is_empty());
    assert!(outside.exists(), "no file was touched");
}

#[tokio::test]
async fn should_list_the_broken_row_once_when_the_ids_come_through_the_event_wrapper() {
    let setup = setup().await;
    broken_row(&setup, "one").await;
    // As the app has it: the storage the use cases get is the wrapper, not the SQLite one.
    let (tx, _events) = tokio::sync::mpsc::unbounded_channel();
    let wrapped: Arc<dyn InstanceStorage> = Arc::new(EventEmittingInstanceStorage::new(
        Arc::new(ChannelEmitter(tx)),
        SqliteInstanceStorage::new(setup.sqlite.pool().clone()),
    ));
    // The file of the unreadable record is kept, as the migration does.
    write_readable_legacy(&setup, "one").await;
    let list = ListBrokenInstancesUseCase::new(
        wrapped.clone(),
        setup.sqlite.clone(),
        Arc::new(LegacyInstanceScanner::new(setup.location_info.clone())),
        setup.location_info.clone(),
    );

    let found = list.execute().await.unwrap();

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(wrapped.stored_ids().await.unwrap(), ["one"]);
}

#[tokio::test]
async fn should_delete_only_the_record_when_another_record_names_the_same_folder() {
    let setup = setup().await;
    // On Windows `Foo` and `foo` are one folder.
    setup.sqlite.upsert(&instance("Foo")).await.unwrap();
    broken_row(&setup, "foo").await;
    let folder = setup
        .location_info
        .instance_dir(&InstanceId::try_from("Foo").unwrap());
    std::fs::create_dir_all(&folder).unwrap();
    let (remove, recording) = remover(&setup);

    remove.execute("foo".to_owned()).await.unwrap();

    assert!(
        recording.0.lock().unwrap().is_empty(),
        "the folder is not removed"
    );
    assert!(folder.exists());
    assert!(
        setup
            .sqlite
            .get(&InstanceId::try_from("Foo").unwrap())
            .await
            .is_ok()
    );
    assert!(setup.list.execute().await.unwrap().is_empty());
}

#[tokio::test]
async fn should_list_a_legacy_file_as_belonging_to_another_instance_when_its_folder_has_another_name()
 {
    let setup = setup().await;
    write_readable_legacy(&setup, "old").await;
    let copy = legacy_file(&setup, "copy");
    std::fs::create_dir_all(copy.parent().unwrap()).unwrap();
    std::fs::copy(legacy_file(&setup, "old"), copy).unwrap();

    let found = listed(&setup).await;

    assert!(found.contains(&(
        "copy".to_owned(),
        BrokenInstanceOrigin::LegacyFile,
        BrokenInstanceReason::LegacyIdMismatch {
            file_id: "old".to_owned()
        }
    )));
}
