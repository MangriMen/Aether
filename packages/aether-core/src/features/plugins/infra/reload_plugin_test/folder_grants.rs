//! Giving a plugin a folder on import.

use std::sync::{Arc, Mutex, atomic::Ordering};

use async_trait::async_trait;

use crate::features::plugins::{
    AllowedPath, AllowedPathRejection, FolderAccess, GrantPluginFolderAccessUseCase, PluginError,
    PluginRegistry, PluginSettings, PluginSettingsStorage, PluginState,
};

use super::common::*;

/// `<temp>/packs/simple/pack.toml`, a pack in a folder of its own.
fn pack_file() -> (tempfile::TempDir, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("packs").join("simple");
    std::fs::create_dir_all(&folder).unwrap();
    let file = folder.join("pack.toml");
    std::fs::write(&file, b"").unwrap();
    std::fs::write(folder.join("index.toml"), b"").unwrap();
    (root, file)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_the_folder_read_only_and_restart_the_plugin_when_a_pack_is_picked() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let old = harness.handle();
    let (_root, file) = pack_file();
    let folder = file.parent().unwrap().to_path_buf();

    let access = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file.clone(), false)
        .await
        .unwrap();

    assert_eq!(
        access,
        FolderAccess::Granted {
            folder: folder.clone()
        }
    );
    assert_eq!(
        settings.folders(),
        [AllowedPath::read_only_folder(&folder)],
        "the folder of the pack, not the file and not the drive"
    );
    assert!(old.is_retired(), "the plugin runs with the folder mounted");
    assert!(noop(&harness.handle()).await.is_ok());

    // Picking the same pack, or another file of that folder, again changes nothing.
    let again = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), folder.join("index.toml"), false)
        .await
        .unwrap();
    assert_eq!(again, FolderAccess::AlreadyAllowed { folder });
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 2);
    assert_eq!(settings.folders().len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_the_folder_when_a_saved_one_over_it_is_left_out_by_the_loader() {
    let harness = Harness::enabled().await;
    let (_root, file) = pack_file();
    // A whole drive saved before it was refused: the loader leaves it out, so it gives the
    // plugin nothing.
    let temp = tempfile::tempdir().unwrap();
    let drive = AllowedPath::read_only_folder(temp.path().ancestors().last().unwrap());
    assert!(drive.exposes(&file));
    let settings = Arc::new(MemoryPluginSettings(Mutex::new(Some(PluginSettings {
        allowed_paths: vec![drive.clone()],
        ..PluginSettings::default()
    }))));
    harness
        .reload_from(&settings)
        .execute(PLUGIN_ID.into())
        .await
        .unwrap();
    assert!(!harness.handle().folders().exposes(&file));
    assert!(!harness.restart_needed(&settings).await);

    let access = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file.clone(), false)
        .await
        .unwrap();

    assert_eq!(
        access,
        FolderAccess::Granted {
            folder: file.parent().unwrap().to_path_buf()
        }
    );
    assert!(harness.handle().folders().exposes(&file));
    assert_eq!(
        settings.folders(),
        [drive, AllowedPath::read_only_folder(file.parent().unwrap())]
    );
    assert!(!harness.restart_needed(&settings).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_no_folder_when_the_plugin_is_busy() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let handle = harness.handle();
    let call = harness.start_long_call(&handle).await;
    let (_root, file) = pack_file();

    let refused = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file, false)
        .await;

    assert!(
        matches!(refused, Err(PluginError::Busy { .. })),
        "{refused:?}"
    );
    assert_eq!(
        settings.folders(),
        [],
        "a folder that did not come with a restart is not left for the next load"
    );
    assert!(!handle.is_retired());
    harness.let_go();
    assert!(call.await.unwrap().is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_keep_the_rights_and_add_the_new_folder_when_a_folder_was_already_given() {
    let harness = Harness::enabled().await;
    let (_root, file) = pack_file();
    let elsewhere = tempfile::tempdir().unwrap();
    let given = AllowedPath {
        read_only: false,
        ..AllowedPath::read_only_folder(elsewhere.path())
    };
    let settings = Arc::new(MemoryPluginSettings(Mutex::new(Some(PluginSettings {
        allowed_paths: vec![given.clone()],
        ..PluginSettings::default()
    }))));

    harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file.clone(), false)
        .await
        .unwrap();

    assert_eq!(
        settings.folders(),
        [given, AllowedPath::read_only_folder(file.parent().unwrap())]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_nothing_when_the_file_is_at_the_top_of_a_drive_or_relative() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    // The top of whatever drive the temporary directory is on, not a drive letter that may
    // not exist.
    let temp = tempfile::tempdir().unwrap();
    let root_file = temp.path().ancestors().last().unwrap().join("pack.toml");

    // Written to lead there through `..`, which `Path::parent` does not follow.
    let back_to_root = temp
        .path()
        .components()
        .filter(|part| matches!(part, std::path::Component::Normal(_)))
        .fold(temp.path().to_path_buf(), |path, _| path.join(".."))
        .join("pack.toml");

    for (file, expected) in [
        (root_file, AllowedPathRejection::HostIsRoot),
        (back_to_root, AllowedPathRejection::HostIsRoot),
        ("pack.toml".into(), AllowedPathRejection::HostNotAbsolute),
    ] {
        let result = harness
            .grant(&settings)
            .execute(PLUGIN_ID.into(), file, false)
            .await;

        assert!(
            matches!(&result, Err(PluginError::InvalidAllowedPath { reason, .. }) if *reason == expected),
            "{result:?}"
        );
    }
    assert_eq!(settings.folders(), []);
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_nothing_when_the_file_is_in_the_plugins_own_cache() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let cache = harness.location_info.plugin_cache_dir(PLUGIN_ID);
    std::fs::create_dir_all(&cache).unwrap();

    for dry_run in [true, false] {
        let result = harness
            .grant(&settings)
            .execute(PLUGIN_ID.into(), cache.join("pack.toml"), dry_run)
            .await;

        assert!(
            matches!(
                &result,
                Err(PluginError::InvalidAllowedPath {
                    reason: AllowedPathRejection::HostOverlapsCore,
                    ..
                })
            ),
            "{result:?}"
        );
    }
    assert_eq!(settings.folders(), []);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_name_the_folder_and_change_nothing_when_it_is_a_dry_run() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let handle = harness.handle();
    let (_root, file) = pack_file();

    let access = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file.clone(), true)
        .await
        .unwrap();

    assert_eq!(
        access,
        FolderAccess::Needed {
            folder: file.parent().unwrap().to_path_buf()
        }
    );
    assert_eq!(settings.folders(), []);
    assert!(!handle.is_retired(), "a dry run restarts nothing");
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 1);

    // A dry run refuses what the grant would refuse, before the user is asked anything.
    let relative = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), "pack.toml".into(), true)
        .await;
    assert!(
        matches!(relative, Err(PluginError::InvalidAllowedPath { .. })),
        "{relative:?}"
    );
}

/// The folder was saved while the plugin ran, as an edit of its settings does: the running
/// instance cannot read it yet.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_restart_the_plugin_when_its_folder_is_saved_but_not_mounted() {
    let harness = Harness::enabled().await;
    let (_root, file) = pack_file();
    let folder = file.parent().unwrap().to_path_buf();
    let settings = Arc::new(MemoryPluginSettings(Mutex::new(Some(PluginSettings {
        allowed_paths: vec![AllowedPath::read_only_folder(&folder)],
        ..PluginSettings::default()
    }))));
    let old = harness.handle();

    let dry_run = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file.clone(), true)
        .await
        .unwrap();
    let access = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file.clone(), false)
        .await
        .unwrap();

    assert_eq!(
        dry_run,
        FolderAccess::Needed {
            folder: folder.clone()
        }
    );
    assert_eq!(access, FolderAccess::Granted { folder });
    assert!(old.is_retired());
    assert!(harness.handle().folders().exposes(&file));
    assert_eq!(
        settings.folders().len(),
        1,
        "the saved folder is not added twice"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_refuse_a_folder_when_the_plugin_is_loading() {
    let harness = Arc::new(Harness::enabled().await);
    let settings = Arc::new(MemoryPluginSettings::default());
    let (_root, file) = pack_file();
    harness.loader.hold_loads.store(true, Ordering::SeqCst);
    let restart = tokio::spawn({
        let harness = Arc::clone(&harness);
        async move { harness.reload().execute(PLUGIN_ID.into()).await }
    });
    harness.loader.load_started.notified().await;

    let dry_run = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file.clone(), true)
        .await;
    let result = harness
        .grant(&settings)
        .execute(PLUGIN_ID.into(), file.clone(), false)
        .await;

    assert!(
        matches!(dry_run, Ok(FolderAccess::Needed { .. })),
        "a folder in no settings is needed whatever the load reads: {dry_run:?}"
    );
    assert!(
        matches!(result, Err(PluginError::LoadingInProgress { .. })),
        "{result:?}"
    );
    assert_eq!(settings.folders(), []);

    // A saved folder may or may not be read by the load: not reported as allowed.
    let saved = Arc::new(MemoryPluginSettings(Mutex::new(Some(PluginSettings {
        allowed_paths: vec![AllowedPath::read_only_folder(file.parent().unwrap())],
        ..PluginSettings::default()
    }))));
    let saved_dry_run = harness
        .grant(&saved)
        .execute(PLUGIN_ID.into(), file, true)
        .await;
    assert!(
        matches!(saved_dry_run, Err(PluginError::LoadingInProgress { .. })),
        "{saved_dry_run:?}"
    );

    harness.loader.load_released.notify_one();
    restart.await.unwrap().unwrap();
}

/// Starts a load of the plugin right after each save, as an enable made at that moment would.
struct StartsLoadingOnSave {
    inner: MemoryPluginSettings,
    plugins: Arc<PluginRegistry>,
}

#[async_trait]
impl PluginSettingsStorage for StartsLoadingOnSave {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
        self.inner.get(plugin_id).await
    }

    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), PluginError> {
        self.inner.upsert(plugin_id, settings).await?;
        self.plugins
            .upsert_with(plugin_id, |plugin| {
                plugin.state = PluginState::Loading;
                Ok(())
            })
            .await
    }
}

/// The load may have read the settings before the folder was saved: telling the user the folder
/// was given would be a guess.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_take_the_folder_back_when_a_load_starts_while_it_is_saved() {
    let harness = Harness::enabled().await;
    harness.disable().execute(PLUGIN_ID.into()).await.unwrap();
    let settings = Arc::new(StartsLoadingOnSave {
        inner: MemoryPluginSettings::default(),
        plugins: Arc::clone(&harness.plugins),
    });
    let (_root, file) = pack_file();
    let grant = GrantPluginFolderAccessUseCase::new(
        Arc::clone(&harness.plugins),
        Arc::clone(&settings) as Arc<dyn PluginSettingsStorage>,
        Arc::new(harness.reload()),
        Arc::clone(&harness.location_info),
    );

    let result = grant.execute(PLUGIN_ID.into(), file, false).await;

    assert!(
        matches!(result, Err(PluginError::LoadingInProgress { .. })),
        "{result:?}"
    );
    assert_eq!(settings.inner.folders(), []);
}
