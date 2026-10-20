use std::sync::Arc;

use chrono::Utc;
use sqlx::SqlitePool;
use sqlx::pool::PoolOptions;

use crate::features::instance::infra::SqliteInstanceStorage;
use crate::features::instance::{
    EditInstance, EditInstanceUseCase, EditInstanceUseCasePort, Instance, InstanceBuilder,
    InstanceId, InstanceInstallStage, InstanceStorage,
};
use crate::features::minecraft::ModLoader;

async fn pool() -> SqlitePool {
    let pool = PoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(":memory:")
                .foreign_keys(true),
        )
        .await
        .unwrap();
    crate::core::migrations::run_migrations(&pool)
        .await
        .unwrap();
    pool
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
    let storage = Arc::new(SqliteInstanceStorage::new(pool().await));
    let id = InstanceId::try_from(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
    storage.upsert(&instance(&id)).await.unwrap();

    let edit = EditInstanceUseCase::new(storage.clone());
    let played = Utc::now();

    let stage_storage = storage.clone();
    let stage_id = id.clone();
    let launch = tokio::spawn(async move {
        stage_storage
            .update(&stage_id, &mut |instance| {
                instance.install_stage = InstanceInstallStage::Installed;
                instance.last_played = Some(played);
                Ok(true)
            })
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
async fn should_not_write_when_update_reports_no_change() {
    let storage = SqliteInstanceStorage::new(pool().await);
    let id = InstanceId::try_from(uuid::Uuid::new_v4().to_string().as_str()).unwrap();
    storage.upsert(&instance(&id)).await.unwrap();

    storage
        .update(&id, &mut |instance| {
            instance.rename("Ignored".to_string())?;
            Ok(false)
        })
        .await
        .unwrap();

    assert_eq!(storage.get(&id).await.unwrap().name(), "Old");
}
