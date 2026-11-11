use crate::features::instance::InstanceName;
use crate::features::minecraft::GameVersion;
use std::sync::Arc;

use chrono::Utc;
use sqlx::SqlitePool;
use sqlx::pool::PoolOptions;
use tempfile::TempDir;

use crate::features::instance::infra::SqliteInstanceStorage;
use crate::features::instance::{
    EditInstance, EditInstanceUseCase, EditInstanceUseCasePort, Instance, InstanceBuilder,
    InstanceId, InstanceInstallStage, InstanceStorage,
};
use crate::features::minecraft::ModLoader;

/// A file database with several connections: the writes really race, so only the storage's own
/// write lock keeps them apart.
async fn pool() -> (TempDir, SqlitePool) {
    let dir = TempDir::new().unwrap();
    let pool = PoolOptions::new()
        .max_connections(4)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(dir.path().join("aether.db"))
                .create_if_missing(true)
                .busy_timeout(std::time::Duration::from_secs(30))
                .foreign_keys(true),
        )
        .await
        .unwrap();
    crate::core::migrations::run_migrations(&pool)
        .await
        .unwrap();
    (dir, pool)
}

fn instance(id: &InstanceId) -> Instance {
    InstanceBuilder::new(
        id.clone(),
        InstanceName::parse("Old").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Fabric,
    )
    .build()
}

fn rename_to(name: &str) -> EditInstance {
    EditInstance {
        name: Some(InstanceName::parse(name).unwrap()),
        override_java_path: None,
        java_path: None,
        override_launch_args: None,
        launch_args: None,
        override_env_vars: None,
        env_vars: None,
        override_memory: None,
        memory: None,
        override_window_settings: None,
        window: None,
        override_hooks: None,
        hooks: None,
    }
}

#[tokio::test]
async fn should_keep_both_changes_when_edit_and_launch_update_run_concurrently() {
    let (_dir, pool) = pool().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool));
    let id = InstanceId::try_from(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
    storage.upsert(&instance(&id)).await.unwrap();

    let edit = EditInstanceUseCase::new(storage.clone());
    let played = Utc::now();

    let stage_storage = storage.clone();
    let stage_id = id.clone();
    let launch = tokio::spawn(async move {
        stage_storage
            .update(
                &stage_id,
                Box::new(|instance| {
                    instance.install_stage = InstanceInstallStage::Installed;
                    instance.last_played = Some(played);
                    Ok(true)
                }),
            )
            .await
            .unwrap();
    });

    edit.execute(id.clone(), rename_to("New")).await.unwrap();
    launch.await.unwrap();

    let stored = storage.get(&id).await.unwrap();
    assert_eq!(stored.name().as_str(), "New");
    assert_eq!(stored.install_stage, InstanceInstallStage::Installed);
    assert!(stored.last_played.is_some());
}

/// The update reads the row, then is held in its change closure while the remove runs; only the
/// write lock taken before the read keeps the remove behind it, so the update cannot write the
/// row back after it was deleted, nor fail for having read a row that changed meanwhile.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_not_resurrect_the_row_when_remove_runs_under_a_concurrent_update() {
    let (_dir, pool) = pool().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool));
    let id = InstanceId::try_from(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
    storage.upsert(&instance(&id)).await.unwrap();

    let (read, mut was_read) = tokio::sync::mpsc::unbounded_channel();
    let updater = storage.clone();
    let update_id = id.clone();
    let update = tokio::spawn(async move {
        updater
            .update(
                &update_id,
                Box::new(move |instance| {
                    let _ = read.send(());
                    // Long enough for a remove that is not kept behind the update to finish.
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    instance.install_stage = InstanceInstallStage::Installed;
                    Ok(true)
                }),
            )
            .await
    });
    was_read.recv().await.unwrap();
    storage.remove(&id).await.unwrap();
    // The remove waited, so the update was applied (not refused as stale) and then deleted.
    update.await.unwrap().expect("the update is applied");

    assert!(storage.get(&id).await.is_err());
}

#[tokio::test]
async fn should_not_write_when_update_reports_no_change() {
    let (_dir, pool) = pool().await;
    let storage = SqliteInstanceStorage::new(pool);
    let id = InstanceId::try_from(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
    storage.upsert(&instance(&id)).await.unwrap();

    storage
        .update(
            &id,
            Box::new(|instance| {
                instance.rename(InstanceName::parse("Ignored").unwrap());
                Ok(false)
            }),
        )
        .await
        .unwrap();

    assert_eq!(storage.get(&id).await.unwrap().name().as_str(), "Old");
}

#[tokio::test]
async fn should_apply_every_update_when_many_run_at_once() {
    const UPDATES: u64 = 16;

    let (_dir, pool) = pool().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool));
    let id = InstanceId::try_from(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
    storage.upsert(&instance(&id)).await.unwrap();

    let tasks: Vec<_> = (0..UPDATES)
        .map(|_| {
            let storage = storage.clone();
            let id = id.clone();
            tokio::spawn(async move {
                storage
                    .update(
                        &id,
                        Box::new(|instance| {
                            instance.time_played += 1;
                            Ok(true)
                        }),
                    )
                    .await
                    .unwrap();
            })
        })
        .collect();
    for task in tasks {
        task.await.unwrap();
    }

    assert_eq!(storage.get(&id).await.unwrap().time_played, UPDATES);
}
