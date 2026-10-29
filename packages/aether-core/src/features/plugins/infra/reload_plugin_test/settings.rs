//! Settings of a running plugin.

use std::sync::Arc;

use crate::features::plugins::{
    AllowedPath, EditPluginSettings, EditPluginSettingsUseCase, FolderMounts,
    GrantPluginHostAccessUseCase, PluginError, PluginSettingsStorage,
};

use super::common::*;

fn edit_folders(folders: Vec<AllowedPath>) -> EditPluginSettings {
    EditPluginSettings {
        allowed_paths: Some(folders),
        ..EditPluginSettings::default()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_need_no_restart_when_a_host_is_edited_on_a_running_plugin() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let handle = harness.handle();

    harness
        .edit(&settings)
        .execute(
            plugin_id(),
            EditPluginSettings {
                allowed_hosts: Some(vec!["example.com".into()]),
                ..EditPluginSettings::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(settings.hosts(), ["example.com"]);
    assert!(!harness.restart_needed(&settings).await);
    assert!(
        !handle.is_retired(),
        "nothing restarts the plugin by itself"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_need_a_restart_until_it_restarts_when_a_folder_is_edited_on_a_running_plugin() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let (first, second) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let first = AllowedPath::read_only_folder(first.path());
    let second = AllowedPath::read_only_folder(second.path());
    let handle = harness.handle();

    harness
        .edit(&settings)
        .execute(plugin_id(), edit_folders(vec![first.clone()]))
        .await
        .unwrap();

    assert!(harness.restart_needed(&settings).await);
    assert!(!handle.is_retired(), "the restart is the user's to ask for");

    // Taking the folder back before the restart leaves the plugin as it runs.
    harness
        .edit(&settings)
        .execute(plugin_id(), edit_folders(vec![]))
        .await
        .unwrap();
    assert!(!harness.restart_needed(&settings).await);

    harness
        .edit(&settings)
        .execute(
            plugin_id(),
            edit_folders(vec![first.clone(), second.clone()]),
        )
        .await
        .unwrap();
    harness
        .reload_from(&settings)
        .execute(plugin_id())
        .await
        .unwrap();

    assert!(!harness.restart_needed(&settings).await);
    assert_eq!(
        *harness.handle().folders(),
        FolderMounts::of(&[first.clone(), second.clone()])
    );

    // The same folders in another order, or one of them twice, mount the same.
    harness
        .edit(&settings)
        .execute(
            plugin_id(),
            edit_folders(vec![second.clone(), first, second]),
        )
        .await
        .unwrap();
    assert!(!harness.restart_needed(&settings).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_refuse_the_restart_and_still_need_it_when_the_plugin_is_busy() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let folder = tempfile::tempdir().unwrap();
    harness
        .edit(&settings)
        .execute(
            plugin_id(),
            edit_folders(vec![AllowedPath::read_only_folder(folder.path())]),
        )
        .await
        .unwrap();
    let handle = harness.handle();
    let call = harness.start_long_call(&handle).await;

    let refused = harness.reload_from(&settings).execute(plugin_id()).await;

    assert!(
        matches!(refused, Err(PluginError::Busy { .. })),
        "{refused:?}"
    );
    assert!(harness.restart_needed(&settings).await);
    assert!(!handle.is_retired());

    harness.let_go();
    assert!(call.await.unwrap().is_ok());
    harness
        .reload_from(&settings)
        .execute(plugin_id())
        .await
        .unwrap();
    assert!(!harness.restart_needed(&settings).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_keep_each_other_when_two_changes_are_made_at_once() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(SlowPluginSettings::default());
    let storage = Arc::clone(&settings) as Arc<dyn PluginSettingsStorage>;
    let folder = tempfile::tempdir().unwrap();
    let folder = AllowedPath::read_only_folder(folder.path());

    let edit = EditPluginSettingsUseCase::new(
        Arc::clone(&harness.plugins),
        Arc::clone(&storage),
        Arc::clone(&harness.location_info),
    );
    let grant = GrantPluginHostAccessUseCase::new(Arc::clone(&harness.plugins), storage);
    let (edited, granted) = tokio::join!(
        edit.execute(plugin_id(), edit_folders(vec![folder.clone()])),
        grant.execute(plugin_id(), "https://example.com/pack.toml".into(), false),
    );

    edited.unwrap();
    granted.unwrap();
    assert_eq!(settings.0.folders(), [folder]);
    assert_eq!(settings.0.hosts(), ["example.com"]);
}
