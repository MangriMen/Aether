use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use tracing::warn;

use crate::{
    features::{
        instance::PluginId,
        plugins::{
            AllowedPathInput, AllowedPathRejection, PluginError, PluginRegistry, PluginSettings,
            PluginSettingsStorage, core_folders,
        },
        settings::LocationInfo,
    },
    shared::io::domain::IoError,
};

use super::ports::EditPluginSettingsUseCasePort;

#[derive(Debug, Default)]
pub struct EditPluginSettings {
    pub allowed_hosts: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<AllowedPathInput>>,
    /// The saved folders that were not read, to keep: any other is deleted from the settings.
    /// An entry here only keeps one that is already saved, it never adds one.
    pub ignored_paths: Option<Vec<AllowedPathInput>>,
}

/// Saves the user's changes to a plugin's settings, running or not. Hosts take effect on the
/// plugin's next request; folders only once it restarts, which is left to the user (the plugin
/// shows as needing a restart until then).
pub(crate) struct EditPluginSettingsUseCase {
    plugin_registry: Arc<PluginRegistry>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    location_info: Arc<LocationInfo>,
}

impl EditPluginSettingsUseCase {
    pub(crate) fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_settings_storage,
            location_info,
        }
    }

    pub(crate) async fn execute(
        &self,
        plugin_id: PluginId,
        edit_settings: EditPluginSettings,
    ) -> Result<PluginSettings, PluginError> {
        let _settings_lock = self.plugin_registry.lock_settings(&plugin_id).await;
        let current = settings_to_edit(self.plugin_settings_storage.as_ref(), &plugin_id)
            .await?
            .unwrap_or_default();
        let core_folders = core_folders(&self.location_info, &plugin_id);
        let merged = apply_edit_changes(&plugin_id, &core_folders, current, edit_settings)?;

        self.plugin_settings_storage
            .upsert(&plugin_id, &merged)
            .await?;

        Ok(merged)
    }
}

/// The saved settings, or none when the file no longer parses: the settings screen is how the
/// user repairs them, so it must open and save over a broken file rather than fail on it.
/// Loading the plugin still reads them strictly and refuses a broken file.
pub(super) async fn settings_to_edit(
    storage: &dyn PluginSettingsStorage,
    plugin_id: &PluginId,
) -> Result<Option<PluginSettings>, PluginError> {
    match storage.get(plugin_id).await {
        Err(PluginError::Storage(IoError::DeserializationError(reason))) => {
            warn!(
                "Settings of plugin '{plugin_id}' are unreadable, editing from defaults: {reason}"
            );
            Ok(None)
        }
        result => result,
    }
}

/// Refuses the whole edit over one bad folder, so that nothing the user did not ask for is
/// saved. A host folder that does not exist is dropped instead: it may be one given earlier
/// and since removed, which should not block every later edit. A file is refused: the user
/// would believe the plugin got a folder it cannot open.
///
/// Only a folder the edit adds is held to that. One already saved that fails a check (it
/// predates the check, or the folder became a file) is dropped with a warning: the settings
/// screen sends every saved folder back with each edit, so refusing it would block every
/// unrelated change, and the loader does not mount it anyway.
fn apply_edit_changes(
    plugin_id: &PluginId,
    core_folders: &[PathBuf],
    mut settings: PluginSettings,
    edit_settings: EditPluginSettings,
) -> Result<PluginSettings, PluginError> {
    if let Some(allowed_hosts) = edit_settings.allowed_hosts {
        settings.allowed_hosts = allowed_hosts;
    }

    if let Some(allowed_paths) = edit_settings.allowed_paths {
        let saved = &settings.allowed_paths;
        let mut kept = Vec::with_capacity(allowed_paths.len());

        for input in allowed_paths {
            let host = input.host.clone();
            let path = input
                .parse()
                .map_err(|reason| PluginError::InvalidAllowedPath {
                    plugin_id: plugin_id.to_owned(),
                    path: host,
                    reason,
                })?;

            let checked = path.validate_against(core_folders).and_then(|()| {
                let host = Path::new(&path.host);
                if host.exists() && !host.is_dir() {
                    Err(AllowedPathRejection::HostNotAFolder)
                } else {
                    Ok(())
                }
            });

            match checked {
                Ok(()) => {}
                Err(reason) if saved.contains(&path) => {
                    warn!(
                        "Plugin '{plugin_id}': dropping saved folder '{}' -> '{}' from its \
                         settings: {reason}",
                        path.host, path.plugin
                    );
                    continue;
                }
                Err(reason) => {
                    return Err(PluginError::InvalidAllowedPath {
                        plugin_id: plugin_id.to_owned(),
                        path: path.host.clone(),
                        reason,
                    });
                }
            }

            if !Path::new(&path.host).exists() {
                warn!(
                    "Plugin '{plugin_id}': dropping folder '{}' -> '{}' from its settings: the \
                     host folder does not exist",
                    path.host, path.plugin
                );
                continue;
            }

            kept.push(path);
        }

        settings.allowed_paths = kept;
    }

    if let Some(keep) = edit_settings.ignored_paths {
        settings.ignored_paths.retain(|ignored| {
            keep.iter()
                .any(|kept| kept.host == ignored.path.host && kept.plugin == ignored.path.plugin)
        });
    }

    Ok(settings)
}

#[async_trait]
impl EditPluginSettingsUseCasePort for EditPluginSettingsUseCase {
    async fn execute(
        &self,
        plugin_id: PluginId,
        edit_settings: EditPluginSettings,
    ) -> Result<PluginSettings, PluginError> {
        self.execute(plugin_id, edit_settings).await
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::features::events::test_support::NoOpEmitter;
    use crate::features::plugins::{AllowedPath, IgnoredAllowedPath};

    use super::*;

    /// Under a directory of its own, so that the core folders are told apart from the test
    /// folders, each made in a temporary directory of its own too.
    struct Launcher {
        _root: TempDir,
        location_info: LocationInfo,
    }

    impl Launcher {
        fn new() -> Self {
            let root = TempDir::new().unwrap();
            let location_info =
                LocationInfo::new(root.path().join("settings"), root.path().join("config"));
            Self {
                _root: root,
                location_info,
            }
        }

        fn edit(&self, paths: Vec<AllowedPathInput>) -> Result<PluginSettings, PluginError> {
            apply_edit_changes(
                &PluginId::from_static("packwiz"),
                &core_folders(&self.location_info, &PluginId::from_static("packwiz")),
                PluginSettings::default(),
                EditPluginSettings {
                    allowed_paths: Some(paths),
                    ..EditPluginSettings::default()
                },
            )
        }
    }

    fn edit(paths: Vec<AllowedPathInput>) -> Result<PluginSettings, PluginError> {
        Launcher::new().edit(paths)
    }

    fn rejection(result: &Result<PluginSettings, PluginError>) -> Option<AllowedPathRejection> {
        match result {
            Err(PluginError::InvalidAllowedPath { reason, .. }) => Some(*reason),
            _ => None,
        }
    }

    fn folder(dir: &Path, plugin: &str) -> AllowedPathInput {
        AllowedPathInput {
            host: dir.to_string_lossy().into_owned(),
            plugin: plugin.into(),
            read_only: true,
        }
    }

    #[test]
    fn should_store_the_plugin_side_under_its_wasi_name_when_the_folder_is_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let host = dir.path().to_string_lossy().into_owned();

        let settings = edit(vec![folder(dir.path(), &host)]).unwrap();

        assert_eq!(
            settings.allowed_paths[0].plugin.as_str(),
            crate::features::plugins::to_wasi_path(&host)
        );
        assert!(settings.allowed_paths[0].read_only);
    }

    #[test]
    fn should_refuse_a_folder_when_it_lies_over_a_core_mount() {
        let dir = tempfile::tempdir().unwrap();

        for plugin in ["/instances", "/cache/x", "/"] {
            let result = edit(vec![folder(dir.path(), plugin)]);

            assert!(
                matches!(
                    result,
                    Err(PluginError::InvalidAllowedPath {
                        reason: AllowedPathRejection::PluginReserved
                            | AllowedPathRejection::PluginIsRoot,
                        ..
                    })
                ),
                "{plugin}: {result:?}"
            );
        }
    }

    #[test]
    fn should_refuse_a_folder_when_its_plugin_path_is_not_plain_names() {
        let dir = tempfile::tempdir().unwrap();

        for plugin in ["/packs/./sub", "/packs/../cache", "/packs/a\0b"] {
            assert_eq!(
                rejection(&edit(vec![folder(dir.path(), plugin)])),
                Some(AllowedPathRejection::PluginNotNormalized),
                "{plugin}"
            );
        }
    }

    /// Names Windows reserves or trims are ordinary folder names elsewhere; the folder is
    /// mounted under its own path, so its name is no reason to refuse it.
    #[cfg(unix)]
    #[test]
    fn should_keep_a_folder_when_its_name_is_special_only_to_windows() {
        let launcher = Launcher::new();
        let dir = tempfile::tempdir().unwrap();
        let folders: Vec<_> = ["aux", "aux.packs", "com1.d", "My Pack ", "pack."]
            .into_iter()
            .map(|name| {
                let folder = dir.path().join(name);
                std::fs::create_dir(&folder).unwrap();
                AllowedPath::read_only_folder(&folder).unwrap()
            })
            .collect();

        let settings = launcher
            .edit(folders.iter().cloned().map(Into::into).collect())
            .unwrap();

        assert_eq!(settings.allowed_paths, folders);
    }

    #[test]
    fn should_save_an_unrelated_edit_when_a_saved_folder_fails_a_check() {
        let launcher = Launcher::new();
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("pack.toml");
        std::fs::write(&file, b"").unwrap();
        let added_dir = tempfile::tempdir().unwrap();

        // Saved before the checks existed, or changed on disk since: a relative host, a folder
        // that is now a file.
        let stale = vec![
            AllowedPath {
                host: "packs".to_owned(),
                plugin: "/packs".into(),
                read_only: true,
            },
            folder(&file, "/file").parse().unwrap(),
        ];
        let added = AllowedPath::read_only_folder(added_dir.path()).unwrap();
        let mut sent: Vec<AllowedPathInput> = stale.iter().cloned().map(Into::into).collect();
        sent.push(added.clone().into());

        let settings = apply_edit_changes(
            &PluginId::from_static("packwiz"),
            &core_folders(&launcher.location_info, &PluginId::from_static("packwiz")),
            PluginSettings {
                allowed_paths: stale,
                ..PluginSettings::default()
            },
            EditPluginSettings {
                allowed_hosts: Some(vec!["example.com".into()]),
                allowed_paths: Some(sent),
                ..EditPluginSettings::default()
            },
        )
        .expect("a stale saved folder must not block the edit");

        assert_eq!(settings.allowed_paths, [added]);
        assert_eq!(settings.allowed_hosts, ["example.com"]);
    }

    #[test]
    fn should_refuse_a_new_folder_when_it_fails_a_check_even_beside_saved_ones() {
        let launcher = Launcher::new();
        let saved_dir = tempfile::tempdir().unwrap();
        let saved = AllowedPath::read_only_folder(saved_dir.path()).unwrap();

        let result = apply_edit_changes(
            &PluginId::from_static("packwiz"),
            &core_folders(&launcher.location_info, &PluginId::from_static("packwiz")),
            PluginSettings {
                allowed_paths: vec![saved.clone()],
                ..PluginSettings::default()
            },
            EditPluginSettings {
                allowed_paths: Some(vec![saved.into(), folder(saved_dir.path(), "/a/../b")]),
                ..EditPluginSettings::default()
            },
        );

        assert_eq!(
            rejection(&result),
            Some(AllowedPathRejection::PluginNotNormalized)
        );
    }

    #[test]
    fn should_refuse_a_folder_when_it_is_the_plugins_cache_instances_or_plugins_folder() {
        let launcher = Launcher::new();
        let location_info = &launcher.location_info;
        let cache = location_info.plugin_cache_dir(&PluginId::from_static("packwiz"));
        let instances = location_info.instances_dir();
        let own_dir = location_info.plugin_dir(&PluginId::from_static("packwiz"));
        let plugins = location_info.plugins_dir();
        for dir in [&cache, &instances, &own_dir] {
            std::fs::create_dir_all(dir).unwrap();
        }

        for host in [&cache, &instances, &own_dir, &plugins] {
            let result = launcher.edit(vec![folder(host, "/mine")]);

            assert!(
                matches!(
                    result,
                    Err(PluginError::InvalidAllowedPath {
                        reason: AllowedPathRejection::HostOverlapsCore,
                        ..
                    })
                ),
                "{}: {result:?}",
                host.display()
            );
        }
    }

    #[test]
    fn should_refuse_a_relative_host_path_when_it_exists_too() {
        let result = edit(vec![AllowedPathInput {
            host: ".".to_owned(),
            plugin: "/here".into(),
            read_only: true,
        }]);

        assert!(matches!(
            result,
            Err(PluginError::InvalidAllowedPath {
                reason: AllowedPathRejection::HostNotAbsolute,
                ..
            })
        ));
    }

    #[test]
    fn should_refuse_a_host_when_it_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("pack.toml");
        std::fs::write(&file, b"").unwrap();

        let result = edit(vec![folder(dir.path(), "/kept"), folder(&file, "/file")]);

        assert_eq!(
            rejection(&result),
            Some(AllowedPathRejection::HostNotAFolder),
            "{result:?}"
        );
    }

    #[test]
    fn should_drop_a_folder_when_it_is_gone() {
        let dir = tempfile::tempdir().unwrap();
        let gone = dir.path().join("gone");

        let settings = edit(vec![folder(&gone, "/gone"), folder(dir.path(), "/kept")]).unwrap();

        assert_eq!(settings.allowed_paths.len(), 1);
        assert_eq!(settings.allowed_paths[0].plugin.as_str(), "/kept");
    }

    /// Settings saved over a file that no longer parses, as the file system storage reports it.
    #[derive(Default)]
    struct BrokenSettingsFile {
        saved: std::sync::Mutex<Option<PluginSettings>>,
    }

    #[async_trait]
    impl PluginSettingsStorage for BrokenSettingsFile {
        async fn get(&self, _plugin_id: &PluginId) -> Result<Option<PluginSettings>, PluginError> {
            match self.saved.lock().unwrap().clone() {
                Some(settings) => Ok(Some(settings)),
                None => Err(PluginError::Storage(IoError::DeserializationError(
                    "expected `=`".into(),
                ))),
            }
        }

        async fn upsert(
            &self,
            _plugin_id: &PluginId,
            settings: &PluginSettings,
        ) -> Result<(), PluginError> {
            *self.saved.lock().unwrap() = Some(settings.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_save_over_the_settings_when_the_saved_file_does_not_parse() {
        let root = TempDir::new().unwrap();
        let storage = Arc::new(BrokenSettingsFile::default());
        let use_case = EditPluginSettingsUseCase::new(
            Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter))),
            Arc::clone(&storage) as Arc<dyn PluginSettingsStorage>,
            Arc::new(LocationInfo::new(
                root.path().join("settings"),
                root.path().join("config"),
            )),
        );

        use_case
            .execute(
                PluginId::from_static("packwiz"),
                EditPluginSettings {
                    allowed_hosts: Some(vec!["example.com".into()]),
                    ..EditPluginSettings::default()
                },
            )
            .await
            .unwrap();

        let saved = storage.saved.lock().unwrap().clone().unwrap();
        assert_eq!(saved.allowed_hosts, ["example.com"]);
        assert_eq!(saved.allowed_paths, []);
    }

    fn ignored(host: &str, plugin: &str) -> IgnoredAllowedPath {
        IgnoredAllowedPath {
            path: AllowedPathInput {
                host: host.into(),
                plugin: plugin.into(),
                read_only: true,
            },
            reason: AllowedPathRejection::PluginReserved,
        }
    }

    fn edit_ignored(keep: Option<Vec<AllowedPathInput>>) -> PluginSettings {
        apply_edit_changes(
            &PluginId::from_static("packwiz"),
            &[],
            PluginSettings {
                ignored_paths: vec![ignored("/a", "/cache"), ignored("/b", "/instances")],
                ..PluginSettings::default()
            },
            EditPluginSettings {
                ignored_paths: keep,
                ..EditPluginSettings::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn should_delete_the_ignored_folders_that_are_not_kept_when_the_edit_lists_some() {
        let settings = edit_ignored(Some(vec![ignored("/b", "/instances").path]));

        assert_eq!(settings.ignored_paths, [ignored("/b", "/instances")]);
    }

    #[test]
    fn should_keep_every_ignored_folder_when_the_edit_does_not_list_them() {
        assert_eq!(edit_ignored(None).ignored_paths.len(), 2);
    }

    #[test]
    fn should_not_add_an_ignored_folder_when_the_edit_lists_one_that_is_not_saved() {
        let settings = edit_ignored(Some(vec![ignored("/c", "/cache").path]));

        assert!(settings.ignored_paths.is_empty());
    }

    #[tokio::test]
    async fn should_show_no_settings_when_the_saved_file_does_not_parse() {
        let storage = BrokenSettingsFile::default();

        let settings = settings_to_edit(&storage, &PluginId::from_static("packwiz"))
            .await
            .unwrap();

        assert!(settings.is_none());
    }
}
