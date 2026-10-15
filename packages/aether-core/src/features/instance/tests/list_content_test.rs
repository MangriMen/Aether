//! `ListContentUseCase` reading an instance's content folders.

use std::sync::Arc;

use tempfile::TempDir;

use crate::features::{
    instance::{ListContentUseCase, infra::SqlitePackStorage},
    settings::LocationInfo,
};

#[tokio::test]
async fn should_list_the_content_there_is_when_some_content_folders_are_missing() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let mods = location_info.instance_dir("instance").join("mods");
    tokio::fs::create_dir_all(&mods).await.unwrap();
    tokio::fs::write(mods.join("a.jar"), b"jar").await.unwrap();
    let use_case = ListContentUseCase::new(
        Arc::new(SqlitePackStorage::in_memory().await),
        location_info,
    );

    let files = use_case.execute("instance".to_owned()).await.unwrap();

    assert_eq!(files.len(), 1);
    assert!(files.contains_key("mods/a.jar"));
}
