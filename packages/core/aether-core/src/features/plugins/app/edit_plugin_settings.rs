use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use tracing::warn;

use crate::{
    features::{
        plugins::{
            AllowedPath, AllowedPathRejection, PluginError, PluginRegistry, PluginSettings,
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
    pub allowed_paths: Option<Vec<AllowedPath>>,
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
        plugin_id: String,
        edit_settings: EditPluginSettings,
    ) -> Result<(), PluginError> {
        let _settings_lock = self.plugin_registry.lock_settings(&plugin_id).await;
        let current = settings_to_edit(self.plugin_settings_storage.as_ref(), &plugin_id)
            .await?
            .unwrap_or_default();
        let core_folders = core_folders(&self.location_info, &plugin_id);
        let merged = apply_edit_changes(&plugin_id, &core_folders, current, edit_settings)?;

        self.plugin_settings_storage
            .upsert(&plugin_id, &merged)
            .await
    }
}

/// The saved settings, or none when the file no longer parses: the settings screen is how the
/// user repairs them, so it must open and save over a broken file rather than fail on it.
/// Loading the plugin still reads them strictly and refuses a broken file.
pub(super) async fn settings_to_edit(
    storage: &dyn PluginSettingsStorage,
    plugin_id: &str,
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
fn apply_edit_changes(
    plugin_id: &str,
    core_folders: &[PathBuf],
    mut settings: PluginSettings,
    edit_settings: EditPluginSettings,
) -> Result<PluginSettings, PluginError> {
    if let Some(allowed_hosts) = edit_settings.allowed_hosts {
        settings.allowed_hosts = allowed_hosts;
    }

    if let Some(allowed_paths) = edit_settings.allowed_paths {
        let mut kept = Vec::with_capacity(allowed_paths.len());

        for path in allowed_paths {
            let path = path.normalized();
            let rejected = |reason| PluginError::InvalidAllowedPath {
                plugin_id: plugin_id.to_owned(),
                path: path.host.clone(),
                reason,
            };

            path.validate_against(core_folders).map_err(rejected)?;

            let host = Path::new(&path.host);
            if !host.exists() {
                continue;
            }
            if !host.is_dir() {
                return Err(rejected(AllowedPathRejection::HostNotAFolder));
            }

            kept.push(path);
        }

        settings.allowed_paths = kept;
    }

    Ok(settings)
}

#[async_trait]
impl EditPluginSettingsUseCasePort for EditPluginSettingsUseCase {
    async fn execute(
        &self,
        plugin_id: String,
        edit_settings: EditPluginSettings,
    ) -> Result<(), PluginError> {
        self.execute(plugin_id, edit_settings).await
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

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

        fn edit(&self, paths: Vec<AllowedPath>) -> Result<PluginSettings, PluginError> {
            apply_edit_changes(
                "packwiz",
                &core_folders(&self.location_info, "packwiz"),
                PluginSettings::default(),
                EditPluginSettings {
                    allowed_paths: Some(paths),
                    ..EditPluginSettings::default()
                },
            )
        }
    }

    fn edit(paths: Vec<AllowedPath>) -> Result<PluginSettings, PluginError> {
        Launcher::new().edit(paths)
    }

    fn rejection(result: &Result<PluginSettings, PluginError>) -> Option<AllowedPathRejection> {
        match result {
            Err(PluginError::InvalidAllowedPath { reason, .. }) => Some(*reason),
            _ => None,
        }
    }

    fn folder(dir: &Path, plugin: &str) -> AllowedPath {
        AllowedPath {
            host: dir.to_string_lossy().into_owned(),
            plugin: plugin.to_owned(),
            read_only: true,
        }
    }

    #[test]
    fn should_store_the_plugin_side_under_its_wasi_name_when_the_folder_is_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let host = dir.path().to_string_lossy().into_owned();

        let settings = edit(vec![folder(dir.path(), &host)]).unwrap();

        assert_eq!(
            settings.allowed_paths[0].plugin,
            crate::features::plugins::to_wasi_path(&host)
        );
        assert!(settings.allowed_paths[0].read_only);
    }

    #[test]
    fn should_refuse_a_folder_when_it_lies_over_a_core_mount() {
        let dir = tempfile::tempdir().unwrap();

        // `/` normalizes to an empty path, refused as not absolute.
        for plugin in ["/instances", "/cache/x", "/"] {
            let result = edit(vec![folder(dir.path(), plugin)]);

            assert!(
                matches!(
                    result,
                    Err(PluginError::InvalidAllowedPath {
                        reason: AllowedPathRejection::PluginReserved
                            | AllowedPathRejection::PluginNotAbsolute,
                        ..
                    })
                ),
                "{plugin}: {result:?}"
            );
        }
    }

    #[test]
    fn should_refuse_a_folder_when_it_is_the_plugins_cache_instances_or_plugins_folder() {
        let launcher = Launcher::new();
        let location_info = &launcher.location_info;
        let cache = location_info.plugin_cache_dir("packwiz");
        let instances = location_info.instances_dir();
        let own_dir = location_info.plugin_dir("packwiz");
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
        let result = edit(vec![AllowedPath {
            host: ".".to_owned(),
            plugin: "/here".to_owned(),
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
    fn should_refuse_a_host_when_it_is_a_whole_drive() {
        // The top of whatever drive the temporary directory is on, not a drive letter that may
        // not exist.
        let temp = TempDir::new().unwrap();
        let root = temp.path().ancestors().last().unwrap();

        let result = edit(vec![folder(root, "/drive")]);

        assert_eq!(
            rejection(&result),
            Some(AllowedPathRejection::HostIsRoot),
            "{result:?}"
        );
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
        assert_eq!(settings.allowed_paths[0].plugin, "/kept");
    }

    /// Settings saved over a file that no longer parses, as the file system storage reports it.
    #[derive(Default)]
    struct BrokenSettingsFile {
        saved: std::sync::Mutex<Option<PluginSettings>>,
    }

    #[async_trait]
    impl PluginSettingsStorage for BrokenSettingsFile {
        async fn get(&self, _plugin_id: &str) -> Result<Option<PluginSettings>, PluginError> {
            match self.saved.lock().unwrap().clone() {
                Some(settings) => Ok(Some(settings)),
                None => Err(PluginError::Storage(IoError::DeserializationError(
                    "expected `=`".into(),
                ))),
            }
        }

        async fn upsert(
            &self,
            _plugin_id: &str,
            settings: &PluginSettings,
        ) -> Result<(), PluginError> {
            *self.saved.lock().unwrap() = Some(settings.clone());
            Ok(())
        }
    }

    struct NoOpEmitter;

    #[async_trait]
    impl crate::features::events::EventEmitter<crate::features::events::Event> for NoOpEmitter {
        async fn emit(
            &self,
            _event: crate::features::events::Event,
        ) -> Result<(), crate::features::events::EventError> {
            Ok(())
        }

        fn listen(
            &self,
            _handler: Box<dyn Fn(crate::features::events::Event) + Send + Sync + 'static>,
        ) {
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
                "packwiz".into(),
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

    #[tokio::test]
    async fn should_show_no_settings_when_the_saved_file_does_not_parse() {
        let storage = BrokenSettingsFile::default();

        let settings = settings_to_edit(&storage, "packwiz").await.unwrap();

        assert!(settings.is_none());
    }
}
