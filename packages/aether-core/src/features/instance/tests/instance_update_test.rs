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
        "Old".to_string(),
        "1.20.1".to_string(),
        ModLoader::Fabric,
    )
    .build()
}

fn rename_to(name: &str) -> EditInstance {
    EditInstance {
        name: Some(name.to_string()),
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
    assert_eq!(stored.name(), "New");
    assert_eq!(stored.install_stage, InstanceInstallStage::Installed);
    assert!(stored.last_played.is_some());
}

#[tokio::test]
async fn should_reject_the_rename_when_the_name_is_only_whitespace() {
    let (_dir, pool) = pool().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool));
    let id = InstanceId::try_from(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
    storage.upsert(&instance(&id)).await.unwrap();

    let result = EditInstanceUseCase::new(storage.clone())
        .execute(id.clone(), rename_to("   "))
        .await;

    assert!(result.is_err());
    assert_eq!(storage.get(&id).await.unwrap().name(), "Old");
}

#[tokio::test]
async fn should_not_resurrect_the_row_when_remove_runs_under_a_concurrent_update() {
    let (_dir, pool) = pool().await;
    let storage = Arc::new(SqliteInstanceStorage::new(pool));
    let id = InstanceId::try_from(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
    storage.upsert(&instance(&id)).await.unwrap();

    let updater = storage.clone();
    let update_id = id.clone();
    let update = tokio::spawn(async move {
        let _ = updater
            .update(
                &update_id,
                Box::new(|instance| {
                    instance.install_stage = InstanceInstallStage::Installed;
                    Ok(true)
                }),
            )
            .await;
    });
    storage.remove(&id).await.unwrap();
    update.await.unwrap();

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
                instance.rename("Ignored".to_string())?;
                Ok(false)
            }),
        )
        .await
        .unwrap();

    assert_eq!(storage.get(&id).await.unwrap().name(), "Old");
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
