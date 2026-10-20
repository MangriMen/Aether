use std::{
    collections::{HashMap, HashSet},
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use tracing::{error, warn};

use crate::{
    features::{
        instance::is_valid_id,
        plugins::{
            ExtractedPlugin, LoadConfig, ManifestError, Plugin, PluginCapabilities, PluginContent,
            PluginError, PluginManifest, PluginState, PluginStateReason, PluginStorage,
            PluginVerificationStorage,
        },
        settings::{LocationInfo, PLUGIN_SETTINGS_FILE_NAME},
    },
    shared::{
        hash::infra::{sha1_async, sha256_async},
        io::domain::IoError,
        io::infra::{
            copy_dir_all, create_dir_all, read_async, read_dir, read_json_async, remove_dir_all,
            rename,
        },
    },
};

use super::{
    super::{
        fs_plugin_source_storage::SOURCE_FILE_NAME,
        fs_plugin_verification_storage::VERIFICATION_FILE_NAME, plugin_utils::hash_failed,
    },
    FsPluginStorageConstants,
};

/// Files the core keeps in a plugin's directory about the plugin. A plugin's archive must never
/// bring its own: `settings.toml` would grant it folders and hosts the user never gave,
/// `verification.json` would pass a local build for a verified release, and `source.json` would
/// point its updates at another source.
const CORE_FILE_NAMES: [&str; 3] = [
    PLUGIN_SETTINGS_FILE_NAME,
    VERIFICATION_FILE_NAME,
    SOURCE_FILE_NAME,
];

/// Copy an unpacked plugin into its directory, leaving out the core's own files.
///
/// Matched case-insensitively: on Windows and macOS `Settings.TOML` is the same file. Every
/// install path (provider release, local import, update) adds a plugin through here, so this is
/// the one place the rule has to hold.
fn copy_plugin_files(source: &Path, target: &Path, plugin_id: &str) -> Result<(), IoError> {
    std::fs::create_dir_all(target)?;

    // What the target already holds, by the exact names it lists.
    let listed = std::fs::read_dir(target)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<HashSet<_>, _>>()?;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();

        let is_core_file = name.to_str().is_some_and(|name| {
            CORE_FILE_NAMES
                .iter()
                .any(|core| name.eq_ignore_ascii_case(core))
        });
        if is_core_file {
            warn!(
                "Plugin '{plugin_id}' brings its own '{}', which belongs to the launcher; left out",
                name.to_string_lossy()
            );
            continue;
        }

        let target_path = target.join(&name);
        // A name the target does not list but resolves is one the file system treats as another
        // (a case-insensitive one folds more than ASCII: `ſettings.toml` is `settings.toml`
        // on APFS), so copying would write over that file, which may be a core's.
        if !listed.contains(&name) && target_path.exists() {
            warn!(
                "Plugin '{plugin_id}' brings '{}', which this file system takes for a file \
                 already there; left out",
                name.to_string_lossy()
            );
            continue;
        }

        if entry.file_type()?.is_dir() {
            copy_dir_all(entry.path(), target_path)?;
        } else {
            std::fs::copy(entry.path(), target_path)?;
        }

        remove_new_core_files(target, plugin_id, &listed)?;
    }

    Ok(())
}

/// Takes out of `target` a core file that a copied entry has just made appear: the name check
/// by [`CORE_FILE_NAMES`] is ASCII only, and the file system decides what else is the same name.
/// A core file `target` listed before the copy is not touched.
fn remove_new_core_files(
    target: &Path,
    plugin_id: &str,
    listed: &HashSet<OsString>,
) -> Result<(), IoError> {
    for core in CORE_FILE_NAMES {
        let path = target.join(core);
        if listed.contains(OsStr::new(core)) || !path.exists() {
            continue;
        }

        warn!(
            "Plugin '{plugin_id}' brings a file this file system takes for its own '{core}', \
             which belongs to the launcher; left out"
        );
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
    }

    Ok(())
}

/// A blocking copy, off the async workers. The extracted plugin goes along: its temporary
/// directory lives until the copy is done, and is removed there too.
async fn copy_extracted(
    extracted_plugin: ExtractedPlugin,
    target_dir: PathBuf,
) -> Result<(), IoError> {
    tokio::task::spawn_blocking(move || {
        let PluginContent::Filesystem { temp_dir } = &extracted_plugin.content;
        copy_plugin_files(temp_dir.path(), &target_dir, &extracted_plugin.plugin_id)
    })
    .await
    .map_err(|err| IoError::from(std::io::Error::other(err)))?
}

async fn remove_dir_if_present(dir: &Path) -> Result<(), IoError> {
    match remove_dir_all(dir).await {
        Err(IoError::IoPathError { source, .. })
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            Ok(())
        }
        result => result,
    }
}

/// The directories a replacement keeps beside the plugin's own start with a dot, which no
/// plugin id does.
fn is_side_dir(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|name| name.starts_with('.'))
}

pub struct FsPluginStorage {
    location_info: Arc<LocationInfo>,
    verification_storage: Arc<dyn PluginVerificationStorage>,
    constants: FsPluginStorageConstants,
}

impl FsPluginStorage {
    pub fn new(
        location_info: Arc<LocationInfo>,
        verification_storage: Arc<dyn PluginVerificationStorage>,
        constants: Option<FsPluginStorageConstants>,
    ) -> Self {
        Self {
            location_info,
            verification_storage,
            constants: constants.unwrap_or_default(),
        }
    }

    /// Beside the plugin's directory, so that moving it in place is a rename on one file system.
    fn staged_dir(&self, plugin_id: &str) -> PathBuf {
        self.location_info
            .plugins_dir()
            .join(format!(".{plugin_id}.staged"))
    }

    fn replaced_dir(&self, plugin_id: &str) -> PathBuf {
        self.location_info
            .plugins_dir()
            .join(format!(".{plugin_id}.replaced"))
    }

    /// Where the plugin in place goes for the moment a restore puts the replaced one back.
    fn trash_dir(&self, plugin_id: &str) -> PathBuf {
        self.location_info
            .plugins_dir()
            .join(format!(".{plugin_id}.trash"))
    }

    fn get_manifest_path(&self, dir: &Path) -> PathBuf {
        dir.join(self.constants.manifest_filename)
    }

    fn get_capabilities_path(&self, dir: &Path) -> PathBuf {
        dir.join(self.constants.capabilities_filename)
    }

    async fn load_manifest(&self, dir: &Path) -> Result<PluginManifest, PluginError> {
        let dto: aether_core_plugin_api::v0::PluginManifestDto =
            read_json_async(&self.get_manifest_path(dir)).await?;
        Ok(dto.try_into()?)
    }

    async fn load_capabilities(
        &self,
        dir: &Path,
    ) -> Result<Option<PluginCapabilities>, PluginError> {
        let dto: Option<super::super::PluginCapabilitiesV1> =
            read_json_async(&self.get_capabilities_path(dir))
                .await
                .map(Some)
                .or_else(|e| {
                    if let Some(io_error) = match &e {
                        IoError::IoPathError { source, .. } | IoError::IoError(source) => {
                            Some(source)
                        }
                        _ => None,
                    } && io_error.kind() == std::io::ErrorKind::NotFound
                    {
                        return Ok(None);
                    }
                    Err(e)
                })?;

        let Some(dto) = dto else {
            return Ok(None);
        };

        let capabilities = PluginCapabilities::from(dto);
        capabilities.validate()?;

        Ok(Some(capabilities))
    }

    /// Absolute path of the binary a plugin is loaded from, per its manifest.
    fn binary_path(dir: &Path, manifest: &PluginManifest) -> PathBuf {
        let LoadConfig::Extism {
            file: relative_file_path,
            ..
        } = &manifest.load;

        dir.join(relative_file_path)
    }

    async fn calc_hash(dir: &Path, manifest: &PluginManifest) -> Result<String, PluginError> {
        let file_content = read_async(Self::binary_path(dir, manifest)).await?;

        sha1_async(file_content)
            .await
            .map_err(|err| hash_failed(&err))
    }

    async fn load_from_dir(&self, dir: &Path) -> Result<Plugin, PluginError> {
        let manifest = self.load_manifest(dir).await?;
        let capabilities = self.load_capabilities(dir).await?;
        let hash = Self::calc_hash(dir, &manifest).await?;
        // An unreadable record lists the plugin as unverified rather than hiding it, so the user
        // can still see and reinstall it. Loading reads the record again and refuses it.
        let verified = match self.verification_storage.get(&manifest.metadata.id).await {
            Ok(verification) => verification.is_some(),
            Err(err) => {
                warn!(
                    "Verification record of plugin '{}' is unreadable, listing it as unverified: {err}",
                    manifest.metadata.id
                );
                false
            }
        };

        Ok(Plugin {
            manifest,
            capabilities,
            hash,
            verified,
            state: PluginState::NotLoaded,
        })
    }
}

impl FsPluginStorage {
    /// Settles the copies a replacement left aside, so that a process killed in the middle of
    /// one does not lose the plugin. Only what is certain is done, since an update may be under
    /// way as this runs:
    ///
    /// - No plugin in place, and a complete one aside: the replacement was cut off between its
    ///   two renames (the only moment the plugin is not in place), so the aside copy goes back.
    /// - A plugin in place whose update was completed (the source record is written last), and
    ///   one aside: the copy aside is only disk space, and goes.
    /// - Anything else is an update still being made, or one cut off after the new files were
    ///   in place and before they were complete: the copy aside is the only whole one, and stays.
    async fn settle_replaced(&self, plugins_dir: &Path) -> Result<(), PluginError> {
        let mut dir_entries = read_dir(plugins_dir).await?;

        while let Some(dir_entry) = dir_entries.next_entry().await.map_err(IoError::from)? {
            let file_name = dir_entry.file_name();
            let Some(plugin_id) = file_name
                .to_str()
                .and_then(|name| name.strip_prefix('.')?.strip_suffix(".replaced"))
                .filter(|id| !id.is_empty())
            else {
                continue;
            };

            let replaced_dir = dir_entry.path();
            let target_dir = self.location_info.plugin_dir(plugin_id);

            let result = if !target_dir.exists() {
                if self.get_manifest_path(&replaced_dir).is_file() {
                    warn!(
                        "Plugin '{plugin_id}' was left aside by an update that was cut off;                          putting it back"
                    );
                    rename(&replaced_dir, &target_dir).await
                } else {
                    Ok(())
                }
            } else if self.is_completely_replaced(&target_dir) {
                remove_dir_if_present(&replaced_dir).await
            } else {
                Ok(())
            };

            if let Err(err) = result {
                warn!("Could not settle the copy of plugin '{plugin_id}' left aside: {err}");
            }
        }

        Ok(())
    }

    /// Whether the plugin in `dir` holds what completing an update writes: the manifest, and
    /// the source record, which is written last.
    fn is_completely_replaced(&self, dir: &Path) -> bool {
        self.get_manifest_path(dir).is_file() && dir.join(SOURCE_FILE_NAME).is_file()
    }

    /// A plugin whose manifest names an id the scheme refuses (see [`PluginMetadata::validate_id`])
    /// is listed as `Failed` with the reason, rather than left out: the user sees it is there,
    /// why it does not run, and can remove it. It never loads: enabling checks the id again.
    ///
    /// Only when the id is the name of its directory, which is the key the plugin is listed
    /// under and the path every operation on it builds; any other plugin stays unlisted.
    async fn load_refused_from_dir(
        &self,
        dir: &Path,
        refusal: ManifestError,
    ) -> Result<Plugin, PluginError> {
        const STAND_IN_ID: &str = "refused-id";

        let mut dto: aether_core_plugin_api::v0::PluginManifestDto =
            read_json_async(&self.get_manifest_path(dir)).await?;
        let dir_matches_id = dir
            .file_name()
            .is_some_and(|name| name.to_str() == Some(dto.metadata.id.as_str()));
        if !dir_matches_id {
            return Err(refusal.into());
        }

        // The rest of the manifest is read as usual, so only the id is taken as it is.
        let id = std::mem::replace(&mut dto.metadata.id, STAND_IN_ID.to_owned());
        let mut manifest = PluginManifest::try_from(dto)?;
        manifest.metadata.id = id;

        Ok(Plugin {
            hash: Self::calc_hash(dir, &manifest).await.unwrap_or_default(),
            manifest,
            capabilities: None,
            verified: false,
            state: PluginState::Failed(PluginStateReason::InvalidManifest {
                details: refusal.to_string(),
            }),
        })
    }
}

#[async_trait]
impl PluginStorage for FsPluginStorage {
    async fn add(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError> {
        let target_dir = self.location_info.plugin_dir(&extracted_plugin.plugin_id);
        create_dir_all(&target_dir).await?;

        Ok(copy_extracted(extracted_plugin, target_dir).await?)
    }

    async fn replace(&self, extracted_plugin: ExtractedPlugin) -> Result<(), PluginError> {
        let plugin_id = extracted_plugin.plugin_id.clone();
        let target_dir = self.location_info.plugin_dir(&plugin_id);
        let staged_dir = self.staged_dir(&plugin_id);
        let replaced_dir = self.replaced_dir(&plugin_id);

        // Leftovers of a replacement that was cut off. Cut off between its two renames, it left
        // the installed plugin only aside: that copy is put back, not thrown away.
        if !target_dir.exists() && replaced_dir.exists() {
            rename(&replaced_dir, &target_dir).await?;
        }
        remove_dir_if_present(&staged_dir).await?;
        remove_dir_if_present(&replaced_dir).await?;

        // The new files are copied in full before the installed plugin is touched; from then on
        // only renames within `plugins/`, which either happen or leave the directory as it was.
        create_dir_all(&staged_dir).await?;
        if let Err(err) = copy_extracted(extracted_plugin, staged_dir.clone()).await {
            remove_dir_if_present(&staged_dir).await.ok();
            return Err(err.into());
        }

        // Nothing installed yet: the plugin is simply put in place, with nothing to keep aside.
        if target_dir.exists()
            && let Err(err) = rename(&target_dir, &replaced_dir).await
        {
            remove_dir_if_present(&staged_dir).await.ok();
            return Err(err.into());
        }

        if let Err(err) = rename(&staged_dir, &target_dir).await {
            if let Err(restore_err) = rename(&replaced_dir, &target_dir).await {
                error!(
                    "Plugin '{plugin_id}' could not be put back after a failed replacement; it is \
                     left in '{}': {restore_err}",
                    replaced_dir.display()
                );
            }
            remove_dir_if_present(&staged_dir).await.ok();
            return Err(err.into());
        }

        Ok(())
    }

    async fn restore_replaced(&self, plugin_id: &str) -> Result<(), PluginError> {
        let target_dir = self.location_info.plugin_dir(plugin_id);
        let replaced_dir = self.replaced_dir(plugin_id);

        if !replaced_dir.exists() {
            return Ok(());
        }

        // The plugin in place is moved aside, not deleted, before the old one takes its place:
        // cut off in between, a plugin is still on disk, and a failed second rename puts it back.
        let trash_dir = self.trash_dir(plugin_id);
        remove_dir_if_present(&trash_dir).await?;
        let moved_aside = target_dir.exists();
        if moved_aside {
            rename(&target_dir, &trash_dir).await?;
        }

        if let Err(err) = rename(&replaced_dir, &target_dir).await {
            if moved_aside && let Err(back_err) = rename(&trash_dir, &target_dir).await {
                error!(
                    "Plugin '{plugin_id}' could not be put back after a failed restore; it is \
                     left in '{}': {back_err}",
                    trash_dir.display()
                );
            }
            return Err(err.into());
        }

        if let Err(err) = remove_dir_if_present(&trash_dir).await {
            warn!(
                "The update of plugin '{plugin_id}' that was undone was left in '{}': {err}",
                trash_dir.display()
            );
        }

        Ok(())
    }

    async fn discard_replaced(&self, plugin_id: &str) -> Result<(), PluginError> {
        Ok(remove_dir_if_present(&self.replaced_dir(plugin_id)).await?)
    }

    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError> {
        let plugins_dir = self.location_info.plugins_dir();

        if !plugins_dir.exists() {
            return Ok(HashMap::new());
        }

        self.settle_replaced(&plugins_dir).await?;

        let mut dir_entries = read_dir(&plugins_dir).await?;
        let mut plugins = HashMap::new();

        while let Some(dir_entry) = dir_entries.next_entry().await.map_err(IoError::from)? {
            let path = dir_entry.path();

            // A replacement under way or cut off: a copy of a plugin listed under its own name,
            // which must not be loaded in its place.
            if is_side_dir(&dir_entry.file_name()) {
                continue;
            }

            let loaded = match self.load_from_dir(&path).await {
                Err(PluginError::Manifest(
                    err @ (ManifestError::InvalidId { .. } | ManifestError::ReservedPluginId),
                )) => self.load_refused_from_dir(&path, err).await,
                other => other,
            };

            match loaded {
                Ok(plugin) => {
                    plugins.insert(plugin.manifest.metadata.id.clone(), plugin);
                }
                Err(e) => {
                    warn!("Failed to load plugin from '{}': {}", path.display(), e);
                }
            }
        }

        Ok(plugins)
    }

    async fn get(&self, plugin_id: &str) -> Result<Plugin, PluginError> {
        let plugin_dir = self.location_info.plugin_dir(plugin_id);
        self.load_from_dir(&plugin_dir).await
    }

    async fn remove(&self, plugin_id: &str) -> Result<(), PluginError> {
        // An id such as "" or ".." would turn the join below into the plugins folder or its parent.
        if !is_valid_id(plugin_id) {
            return Err(PluginError::NotFound {
                plugin_id: plugin_id.to_owned(),
            });
        }

        let plugin_dir = self.location_info.plugin_dir(plugin_id);

        // The copies of an update stay behind otherwise, and `settle_replaced` would put the
        // removed plugin back from `.replaced` at the next listing.
        for side_dir in [
            self.staged_dir(plugin_id),
            self.replaced_dir(plugin_id),
            self.trash_dir(plugin_id),
        ] {
            remove_dir_if_present(&side_dir).await?;
        }

        if !plugin_dir.exists() {
            return Ok(());
        }

        remove_dir_all(plugin_dir).await?;

        Ok(())
    }

    async fn wasm_sha256(&self, plugin_id: &str) -> Result<String, PluginError> {
        let plugin_dir = self.location_info.plugin_dir(plugin_id);
        let manifest = self.load_manifest(&plugin_dir).await?;
        let file_content = read_async(Self::binary_path(&plugin_dir, &manifest)).await?;

        sha256_async(file_content)
            .await
            .map_err(|err| hash_failed(&err))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::features::plugins::infra::FsPluginVerificationStorage;

    const MANIFEST: &str = r#"{
        "metadata": {
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "0.1.0",
            "description": null,
            "authors": [],
            "license": null
        },
        "runtime": { "allowedHosts": [], "allowedPaths": [] },
        "load": { "type": "extism", "file": "plugin.wasm", "memoryLimit": null },
        "api": { "version": "*", "features": [] }
    }"#;

    struct Fixture {
        _temp: TempDir,
        location_info: Arc<LocationInfo>,
        storage: FsPluginStorage,
    }

    impl Fixture {
        /// `test-plugin` 0.1.0 installed, with a settings file of the core's.
        fn installed() -> Self {
            let temp = TempDir::new().unwrap();
            let location_info = Arc::new(LocationInfo::new(
                temp.path().join("settings"),
                temp.path().join("config"),
            ));
            write_plugin(&location_info.plugin_dir("test-plugin"), "0.1.0");
            std::fs::write(
                location_info.plugin_settings("test-plugin"),
                b"allowed_hosts = []\n",
            )
            .unwrap();
            let storage = FsPluginStorage::new(
                location_info.clone(),
                Arc::new(FsPluginVerificationStorage::new(location_info.clone())),
                None,
            );
            Self {
                _temp: temp,
                location_info,
                storage,
            }
        }

        fn plugin_dir(&self) -> PathBuf {
            self.location_info.plugin_dir("test-plugin")
        }

        fn plugins_dir_entries(&self) -> Vec<String> {
            std::fs::read_dir(self.location_info.plugins_dir())
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        }
    }

    fn write_plugin(dir: &Path, version: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("manifest.json"),
            MANIFEST.replace("0.1.0", version),
        )
        .unwrap();
        std::fs::write(dir.join("plugin.wasm"), version).unwrap();
    }

    fn write_plugin_with_id(dir: &Path, id: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("manifest.json"),
            MANIFEST.replace("\"test-plugin\"", &format!("{id:?}")),
        )
        .unwrap();
        std::fs::write(dir.join("plugin.wasm"), "0.1.0").unwrap();
    }

    fn extracted(version: &str) -> ExtractedPlugin {
        let temp_dir = TempDir::new().unwrap();
        write_plugin(temp_dir.path(), version);
        let dto: aether_core_plugin_api::v0::PluginManifestDto =
            serde_json::from_str(&MANIFEST.replace("0.1.0", version)).unwrap();
        ExtractedPlugin {
            plugin_id: "test-plugin".into(),
            manifest: dto.try_into().unwrap(),
            content: PluginContent::Filesystem { temp_dir },
        }
    }

    #[tokio::test]
    async fn should_list_the_plugin_as_failed_when_its_id_is_outside_the_scheme() {
        let fixture = Fixture::installed();
        for id in ["Packwiz", "My Plugin", "aether"] {
            write_plugin_with_id(&fixture.location_info.plugin_dir(id), id);
        }

        let plugins = fixture.storage.list().await.unwrap();

        for id in ["Packwiz", "My Plugin", "aether"] {
            let plugin = plugins
                .get(id)
                .unwrap_or_else(|| panic!("'{id}' stays listed"));
            assert_eq!(plugin.manifest.metadata.id, id);
            assert!(
                matches!(
                    &plugin.state,
                    PluginState::Failed(PluginStateReason::InvalidManifest { .. })
                ),
                "{id}"
            );
        }
        assert!(plugins.contains_key("test-plugin"));
    }

    #[tokio::test]
    async fn should_not_list_the_plugin_when_its_invalid_id_is_not_its_directory() {
        let fixture = Fixture::installed();
        let dir = fixture.location_info.plugins_dir().join("other");
        write_plugin_with_id(&dir, "Packwiz");

        let plugins = fixture.storage.list().await.unwrap();

        assert_eq!(plugins.len(), 1);
        assert!(plugins.contains_key("test-plugin"));
    }

    #[tokio::test]
    async fn should_list_the_plugin_as_unverified_when_its_verification_record_is_corrupt() {
        let fixture = Fixture::installed();
        std::fs::write(
            fixture.plugin_dir().join(VERIFICATION_FILE_NAME),
            b"{\"wasmSha2",
        )
        .unwrap();

        let plugins = fixture.storage.list().await.unwrap();

        let plugin = plugins.get("test-plugin").expect("the plugin stays listed");
        assert!(!plugin.verified);
    }

    #[tokio::test]
    async fn should_keep_the_replaced_plugin_aside_when_it_is_not_restored_yet() {
        let fixture = Fixture::installed();

        fixture.storage.replace(extracted("0.2.0")).await.unwrap();
        assert_eq!(
            std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
            b"0.2.0"
        );
        assert!(
            !fixture
                .plugin_dir()
                .join(PLUGIN_SETTINGS_FILE_NAME)
                .exists()
        );

        fixture
            .storage
            .restore_replaced("test-plugin")
            .await
            .unwrap();

        assert_eq!(
            std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
            b"0.1.0"
        );
        assert!(
            fixture
                .plugin_dir()
                .join(PLUGIN_SETTINGS_FILE_NAME)
                .exists()
        );
        assert_eq!(fixture.plugins_dir_entries(), ["test-plugin"]);
    }

    #[tokio::test]
    async fn should_drop_the_replaced_plugin_when_it_is_discarded() {
        let fixture = Fixture::installed();

        fixture.storage.replace(extracted("0.2.0")).await.unwrap();
        fixture
            .storage
            .discard_replaced("test-plugin")
            .await
            .unwrap();

        assert_eq!(
            std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
            b"0.2.0"
        );
        assert_eq!(fixture.plugins_dir_entries(), ["test-plugin"]);
    }

    #[tokio::test]
    async fn should_keep_the_installed_plugin_when_the_new_files_cannot_be_copied() {
        let fixture = Fixture::installed();
        let broken = extracted("0.2.0");
        let PluginContent::Filesystem { temp_dir } = &broken.content;
        std::fs::remove_dir_all(temp_dir.path()).unwrap();

        let result = fixture.storage.replace(broken).await;

        assert!(result.is_err());
        assert_eq!(
            std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
            b"0.1.0"
        );
        assert_eq!(fixture.plugins_dir_entries(), ["test-plugin"]);
    }

    #[tokio::test]
    async fn should_refuse_to_remove_a_plugin_by_an_id_that_leaves_the_plugins_folder() {
        let fixture = Fixture::installed();

        for id in ["", "..", "../test-plugin", "/abs"] {
            assert!(
                matches!(
                    fixture.storage.remove(id).await,
                    Err(PluginError::NotFound { .. })
                ),
                "{id:?}"
            );
        }

        assert!(fixture.plugin_dir().exists());
    }

    #[tokio::test]
    async fn should_not_bring_a_removed_plugin_back_when_an_update_copy_was_left_aside() {
        let fixture = Fixture::installed();
        fixture.storage.replace(extracted("0.2.0")).await.unwrap();

        fixture.storage.remove("test-plugin").await.unwrap();
        let plugins = fixture.storage.list().await.unwrap();

        assert!(plugins.is_empty());
        assert!(fixture.plugins_dir_entries().is_empty());
    }

    #[tokio::test]
    async fn should_list_the_installed_plugin_when_a_replaced_copy_is_beside_it() {
        let fixture = Fixture::installed();
        fixture.storage.replace(extracted("0.2.0")).await.unwrap();

        let plugins = fixture.storage.list().await.unwrap();

        assert_eq!(plugins.len(), 1);
        assert_eq!(
            plugins["test-plugin"].manifest.metadata.version,
            semver::Version::new(0, 2, 0)
        );
    }

    #[tokio::test]
    async fn should_put_the_plugin_back_when_a_replacement_was_cut_off_between_its_renames() {
        let fixture = Fixture::installed();
        fixture.storage.replace(extracted("0.2.0")).await.unwrap();
        std::fs::remove_dir_all(fixture.plugin_dir()).unwrap();

        fixture.storage.replace(extracted("0.3.0")).await.unwrap();
        fixture
            .storage
            .restore_replaced("test-plugin")
            .await
            .unwrap();

        assert_eq!(
            std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
            b"0.1.0"
        );
    }

    #[test]
    fn should_remove_the_core_file_when_the_file_system_takes_a_copied_name_for_it() {
        let dir = TempDir::new().unwrap();
        // Not the exact name the core uses, which the ASCII check would already have left out.
        // Only a case-insensitive file system makes this one `settings.toml`.
        std::fs::write(dir.path().join("Settings.TOML"), b"allowed_hosts = [\"*\"]").unwrap();

        remove_new_core_files(dir.path(), "test-plugin", &HashSet::new()).unwrap();

        assert!(!dir.path().join(PLUGIN_SETTINGS_FILE_NAME).exists());
    }

    #[test]
    fn should_keep_the_core_file_when_it_was_there_before_the_copy() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join(PLUGIN_SETTINGS_FILE_NAME),
            b"allowed_hosts = []",
        )
        .unwrap();
        let listed = HashSet::from([OsString::from(PLUGIN_SETTINGS_FILE_NAME)]);

        remove_new_core_files(dir.path(), "test-plugin", &listed).unwrap();

        assert!(dir.path().join(PLUGIN_SETTINGS_FILE_NAME).exists());
    }

    #[test]
    fn should_leave_out_the_core_files_when_the_plugin_brings_them_in_another_case() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        write_plugin(source.path(), "0.1.0");
        for name in ["Settings.toml", "VERIFICATION.JSON", "Source.Json"] {
            std::fs::write(source.path().join(name), b"{}").unwrap();
        }

        copy_plugin_files(source.path(), target.path(), "test-plugin").unwrap();

        let mut names: Vec<_> = std::fs::read_dir(target.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["manifest.json", "plugin.wasm"]);
    }

    /// What a process killed between the two renames of a replacement leaves.
    fn cut_off_between_the_renames(fixture: &Fixture) {
        std::fs::rename(
            fixture.plugin_dir(),
            fixture
                .location_info
                .plugins_dir()
                .join(".test-plugin.replaced"),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn should_list_the_plugin_with_its_settings_when_a_replacement_was_cut_off_between_its_renames()
     {
        let fixture = Fixture::installed();
        cut_off_between_the_renames(&fixture);

        let plugins = fixture.storage.list().await.unwrap();

        assert!(plugins.contains_key("test-plugin"));
        assert!(
            fixture
                .plugin_dir()
                .join(PLUGIN_SETTINGS_FILE_NAME)
                .exists()
        );
        assert_eq!(fixture.plugins_dir_entries(), ["test-plugin"]);
    }

    #[tokio::test]
    async fn should_drop_the_replaced_copy_when_listing_finds_the_update_completed() {
        let fixture = Fixture::installed();
        fixture.storage.replace(extracted("0.2.0")).await.unwrap();
        // The source record is what completing an update writes last.
        std::fs::write(fixture.plugin_dir().join(SOURCE_FILE_NAME), b"{}").unwrap();

        let plugins = fixture.storage.list().await.unwrap();

        assert_eq!(
            plugins["test-plugin"].manifest.metadata.version,
            semver::Version::new(0, 2, 0)
        );
        assert_eq!(fixture.plugins_dir_entries(), ["test-plugin"]);
    }

    #[tokio::test]
    async fn should_keep_the_replaced_copy_when_listing_finds_the_update_not_completed() {
        let fixture = Fixture::installed();
        fixture.storage.replace(extracted("0.2.0")).await.unwrap();

        fixture.storage.list().await.unwrap();

        // It may be an update under way, or one cut off before the new files were complete:
        // either way the copy aside is what a rollback has.
        fixture
            .storage
            .restore_replaced("test-plugin")
            .await
            .unwrap();
        assert_eq!(
            std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
            b"0.1.0"
        );
    }

    #[tokio::test]
    async fn should_restore_the_replaced_plugin_when_none_is_in_place() {
        let fixture = Fixture::installed();
        cut_off_between_the_renames(&fixture);

        fixture
            .storage
            .restore_replaced("test-plugin")
            .await
            .unwrap();

        assert_eq!(
            std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
            b"0.1.0"
        );
        assert_eq!(fixture.plugins_dir_entries(), ["test-plugin"]);
    }

    #[tokio::test]
    async fn should_restore_the_replaced_plugin_when_a_restore_left_its_trash_behind() {
        let fixture = Fixture::installed();
        fixture.storage.replace(extracted("0.2.0")).await.unwrap();
        let trash = fixture
            .location_info
            .plugins_dir()
            .join(".test-plugin.trash");
        write_plugin(&trash, "0.0.1");

        fixture
            .storage
            .restore_replaced("test-plugin")
            .await
            .unwrap();

        assert_eq!(
            std::fs::read(fixture.plugin_dir().join("plugin.wasm")).unwrap(),
            b"0.1.0"
        );
        assert_eq!(fixture.plugins_dir_entries(), ["test-plugin"]);
    }
}
