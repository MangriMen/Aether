use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, Weak},
};

use async_trait::async_trait;
use extism::{Function, Manifest, Plugin, PluginBuilder, Wasm};

use crate::{
    core::app::AetherContainer,
    features::{
        plugins::{
            FolderMounts, LoadConfig, ManifestError, PathMapping, PluginError, PluginHandle,
            PluginInternalEvent, PluginLoader, PluginManifest, PluginSettings,
            PluginVerificationStorage, core_folders, partition_mountable,
        },
        settings::LocationInfo,
    },
    shared::{
        hash::infra::sha256_async,
        io::infra::{create_dir_all, read_async, write_toml_async},
        limits::PluginSandboxLimits,
    },
};

use super::{
    extism_limits::{DEFAULT_TIMEOUT, bytes_to_pages},
    host_functions::{get_host_functions, host_function_names},
    models::{ExtismPluginInstance, get_default_cache_config},
};

use super::super::plugin_utils::{
    get_default_allowed_paths, hash_failed, is_read_only_plugin_mount, to_extism_mount_key,
};

pub struct ExtismPluginLoader {
    location_info: Arc<LocationInfo>,
    verification_storage: Arc<dyn PluginVerificationStorage>,
    container: OnceLock<Weak<AetherContainer>>,
    limits: PluginSandboxLimits,
}

impl ExtismPluginLoader {
    pub fn new(
        location_info: Arc<LocationInfo>,
        verification_storage: Arc<dyn PluginVerificationStorage>,
        limits: PluginSandboxLimits,
    ) -> Self {
        Self {
            location_info,
            verification_storage,
            limits,
            container: OnceLock::new(),
        }
    }

    pub fn set_container(&self, container: &Arc<AetherContainer>) {
        let _ = self.container.set(Arc::downgrade(container));
    }

    async fn ensure_cache_config_file(&self) -> Result<PathBuf, PluginError> {
        let cache_config_path = self.location_info.wasm_cache_config();

        if !cache_config_path.exists() {
            let cache_dir = self.location_info.wasm_cache_dir();
            create_dir_all(&cache_dir).await?;

            write_toml_async(&cache_config_path, get_default_cache_config(cache_dir)).await?;
        }

        Ok(cache_config_path)
    }

    async fn ensure_default_allowed_paths(
        &self,
        plugin_id: &str,
    ) -> Result<HashMap<String, PathBuf>, PluginError> {
        let default_allowed_paths = get_default_allowed_paths(&self.location_info, plugin_id);

        for host in default_allowed_paths.keys() {
            create_dir_all(host).await?;
        }

        Ok(default_allowed_paths)
    }

    fn resolve_absolute_wasm_path(&self, plugin_id: &str, wasm_file: &Path) -> PathBuf {
        self.location_info.plugin_dir(plugin_id).join(wasm_file)
    }

    /// Splits an Extism `LoadConfig` into the absolute wasm path and its memory limit.
    fn extism_load_config(&self, manifest: &PluginManifest) -> (PathBuf, usize) {
        let LoadConfig::Extism { file, memory_limit } = &manifest.load;

        (
            self.resolve_absolute_wasm_path(&manifest.metadata.id, file),
            memory_limit.unwrap_or(self.limits.default_memory_limit_bytes),
        )
    }

    /// Matches the wasm blob on disk against the trust-on-first-use record and returns the
    /// digest to pin in the Extism manifest; `None` for a plugin without a record (a local
    /// import), which loads unverified.
    ///
    /// Checked here as well as through `Wasm::with_hash`: this pass gives the user an
    /// actionable error, `with_hash` closes the gap between this read and Extism's own.
    async fn resolve_expected_wasm_sha256(
        &self,
        plugin_id: &str,
        wasm_path: &Path,
    ) -> Result<Option<String>, PluginError> {
        let Some(verification) = self.verification_storage.get(plugin_id).await? else {
            tracing::warn!(
                "Plugin '{plugin_id}' is not verified: no wasm sha256 was recorded for it, \
                 because it was not installed through a plugin provider. \
                 Loading it without an integrity check."
            );
            return Ok(None);
        };

        let actual_sha256 = sha256_async(read_async(wasm_path).await?)
            .await
            .map_err(|err| hash_failed(&err))?;

        if actual_sha256 != verification.wasm_sha256 {
            let reason = format!(
                "wasm integrity check failed: '{}' hashes to {} but {} was recorded when the \
                 plugin was installed from {}. The plugin binary changed on disk — reinstall \
                 the plugin from its source before enabling it.",
                wasm_path.display(),
                actual_sha256,
                verification.wasm_sha256,
                verification.source_type,
            );
            tracing::error!("Refusing to load plugin '{plugin_id}': {reason}");

            return Err(PluginError::LoadFailed {
                plugin_id: plugin_id.to_owned(),
                reason,
            });
        }

        Ok(Some(verification.wasm_sha256))
    }

    fn build_wasm_manifest(
        &self,
        manifest: &PluginManifest,
        allowed_paths: Vec<PathMapping>,
        expected_wasm_sha256: Option<String>,
    ) -> Manifest {
        let (wasm_file_path, memory_limit_bytes) = self.extism_load_config(manifest);

        let mut wasm_file = Wasm::file(wasm_file_path);
        if let Some(sha256) = expected_wasm_sha256 {
            // Extism checks it on the bytes it reads, before wasmtime sees them, so the
            // compilation cache cannot serve a module for a file that no longer matches.
            wasm_file = wasm_file.with_hash(sha256);
        }

        // No `with_allowed_hosts`, so Extism's built-in `http_request` refuses every call: it
        // checks only the first URL and follows redirects unchecked. The network is reachable
        // only through the host functions, which vet each hop.
        Manifest::new([wasm_file])
            .with_allowed_paths(allowed_paths.into_iter().map(Into::into))
            .with_memory_max(bytes_to_pages(memory_limit_bytes))
            .with_timeout(DEFAULT_TIMEOUT)
    }

    /// Links exactly `host_functions`: wasmtime refuses a module with an unsatisfied import, so
    /// a plugin importing a function it did not declare fails here, not on its first call.
    fn build_plugin(
        plugin_id: &str,
        wasm_manifest: &Manifest,
        cache_dir: Option<&PathBuf>,
        host_functions: Vec<Function>,
        fuel_limit: u64,
    ) -> Result<Plugin, PluginError> {
        let mut builder = PluginBuilder::new(wasm_manifest)
            .with_functions(host_functions)
            .with_wasi(true)
            .with_fuel_limit(fuel_limit);

        if let Some(cache_dir) = cache_dir {
            builder = builder.with_cache_config(cache_dir);
        }

        builder.build().map_err(|e| {
            tracing::debug!("Load failed for plugin {}: {:#}", plugin_id, e);
            PluginError::LoadFailed {
                plugin_id: plugin_id.to_owned(),
                reason: describe_build_error(&e),
            }
        })
    }
}

#[async_trait]
impl PluginLoader for ExtismPluginLoader {
    async fn load(
        &self,
        manifest: &PluginManifest,
        settings: Option<&PluginSettings>,
    ) -> Result<PluginHandle, PluginError> {
        let plugin_id = &manifest.metadata.id;

        // The enable use cases already refuse such a manifest; without a declaration there is
        // nothing to link, so this is a refusal rather than a fallback.
        let Some(declared_host_functions) = manifest.api.host_functions.as_deref() else {
            return Err(PluginError::Manifest(
                ManifestError::HostFunctionsNotDeclared,
            ));
        };

        // First: a binary that no longer matches what was installed must not reach the runtime.
        let (wasm_path, _) = self.extism_load_config(manifest);
        let expected_wasm_sha256 = self
            .resolve_expected_wasm_sha256(plugin_id, &wasm_path)
            .await?;

        let cache_config = self.ensure_cache_config_file().await?;
        let default_allowed_paths = self.ensure_default_allowed_paths(plugin_id).await?;

        let container = self
            .container
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| PluginError::LoadFailed {
                plugin_id: plugin_id.to_owned(),
                reason: "the application container is not available".to_owned(),
            })?;

        let core_folders = core_folders(&self.location_info, plugin_id);
        let allowed_paths = resolve_allowed_paths(
            manifest,
            settings,
            Some(&default_allowed_paths),
            &core_folders,
        );
        // What the plugin is built with, told to whoever asks what it can open.
        let mounted_folders = FolderMounts::of(
            partition_mountable(
                settings.map_or(&[][..], |settings| settings.allowed_paths.as_slice()),
                &core_folders,
            )
            .0,
        );

        let wasm_manifest = self.build_wasm_manifest(manifest, allowed_paths, expected_wasm_sha256);

        let host_functions = get_host_functions(plugin_id, &container, declared_host_functions);

        // Compiling the module is seconds of CPU work, kept off the runtime workers just like
        // the calls into the plugin (see `PluginHandle`).
        let fuel_limit = self.limits.default_fuel_limit;
        let extism_plugin = tokio::task::spawn_blocking({
            let plugin_id = plugin_id.clone();
            move || {
                Self::build_plugin(
                    &plugin_id,
                    &wasm_manifest,
                    Some(&cache_config),
                    host_functions,
                    fuel_limit,
                )
            }
        })
        .await
        .unwrap_or_else(|err| std::panic::resume_unwind(err.into_panic()))?;

        let plugin = PluginHandle::new(ExtismPluginInstance::new(extism_plugin, plugin_id.clone()))
            .with_folders(mounted_folders);
        if let Err(err) = plugin
            .run("handle_event", |instance| {
                instance.handle_event(&PluginInternalEvent::Loaded)
            })
            .await
            .flatten()
        {
            tracing::debug!(
                "Failed to call on_load on plugin {}: {:?}",
                plugin.id(),
                err
            );
        }

        Ok(plugin)
    }

    async fn unload(&self, instance: PluginHandle) -> Result<(), PluginError> {
        // A failing `on_unload` does not keep the plugin loaded: the instance is gone either way.
        if let Err(err) = instance
            .retire(|plugin| plugin.handle_event(&PluginInternalEvent::Unloaded))
            .await?
        {
            tracing::debug!(
                "Failed to call on_unload on plugin {}: {:?}",
                instance.id(),
                err
            );
        }

        Ok(())
    }

    fn host_functions(&self) -> Vec<&'static str> {
        host_function_names()
    }
}

/// Turns wasmtime's report of an unsatisfied host function import into something a plugin
/// author can act on; any other build error is passed through as is.
fn describe_build_error(error: &extism::Error) -> String {
    let message = format!("{error:#}");

    let Some(name) = unlinked_host_function(&message) else {
        return message;
    };

    if host_function_names().contains(&name) {
        format!(
            "the plugin's wasm imports host function `{name}`, but its manifest does not \
             declare it in `api.hostFunctions`"
        )
    } else {
        format!(
            "the plugin's wasm imports host function `{name}`, which its manifest does not \
             declare in `api.hostFunctions` and this launcher does not provide; the plugin \
             was probably built for a newer version of the launcher"
        )
    }
}

/// The name in wasmtime's "unknown import: `extism:host/user::<name>` has not been defined"
/// (`UnknownImportError`); `extism:host/user` is Extism's default host function namespace.
fn unlinked_host_function(message: &str) -> Option<&str> {
    const PREFIX: &str = "unknown import: `extism:host/user::";

    let start = message.find(PREFIX)? + PREFIX.len();
    let len = message[start..].find('`')?;

    Some(&message[start..start + len])
}

/// The folders mounted into the plugin: the core's own, then those the user gave it.
///
/// The manifest's `runtime.allowedPaths` is ignored: Extism opens relative host paths against
/// the launcher's working directory, so a manifest naming `../..` would get the launcher's
/// folders mounted read-write without the user being asked.
///
/// `core_folders` are the host folders a folder of the user's must stay clear of.
fn resolve_allowed_paths(
    manifest: &PluginManifest,
    settings: Option<&PluginSettings>,
    default_allowed_paths: Option<&HashMap<String, PathBuf>>,
    core_folders: &[PathBuf],
) -> Vec<PathMapping> {
    if !manifest.runtime.allowed_paths.is_empty() {
        tracing::warn!(
            "Plugin '{}' declares `runtime.allowedPaths` in its manifest; they are ignored. \
             Folders are given to a plugin by the user, in its settings.",
            manifest.metadata.id
        );
    }

    let mut allowed_paths = Vec::new();

    if let Some(default_allowed_paths) = default_allowed_paths {
        // `/instances` gets Extism's `ro:` prefix, so WASI can only read it. The map itself
        // stays unprefixed: directories are created and `#`-paths resolved through it.
        allowed_paths.extend(default_allowed_paths.iter().map(|(host, plugin)| {
            PathMapping(
                to_extism_mount_key(host, is_read_only_plugin_mount(plugin)),
                plugin.clone(),
            )
        }));
    }

    if let Some(settings) = settings {
        // The settings file is not only written by the edits that check a folder: it may predate
        // a check, or have been changed by hand. A relative host would be opened against the
        // launcher's working directory; a plugin path over `/cache` or `/instances`, or a host
        // over a core folder, would take that mount's place in Extism's map or open `plugins/`
        // to the plugin. Such a folder is left out, and the plugin loads without it.
        let (kept, refused) = partition_mountable(&settings.allowed_paths, core_folders);
        for (path, reason) in refused {
            tracing::warn!(
                "Plugin '{}': folder '{}' -> '{}' is not mounted: {reason}.",
                manifest.metadata.id,
                path.host,
                path.plugin
            );
        }

        allowed_paths.extend(kept.into_iter().map(|path| {
            PathMapping(
                to_extism_mount_key(&path.host, path.read_only),
                PathBuf::from(&path.plugin),
            )
        }));
    }

    allowed_paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::plugins::domain::{ApiConfig, PluginMetadata, RuntimeConfig};

    use crate::features::plugins::{AllowedPath, PluginSourceType, PluginVerification};

    use extism::UserData;

    use super::super::{
        extism_limits::WASM_PAGE_SIZE_BYTES, host_functions::select_host_functions,
    };

    /// In-memory stand-in for `FsPluginVerificationStorage`.
    #[derive(Default)]
    struct StubVerificationStorage(Option<PluginVerification>);

    #[async_trait]
    impl PluginVerificationStorage for StubVerificationStorage {
        async fn save(&self, _: &str, _: &PluginVerification) -> Result<(), PluginError> {
            Ok(())
        }

        async fn get(&self, _: &str) -> Result<Option<PluginVerification>, PluginError> {
            Ok(self.0.clone())
        }

        async fn remove(&self, _: &str) -> Result<(), PluginError> {
            Ok(())
        }
    }

    fn loader() -> ExtismPluginLoader {
        loader_at(
            LocationInfo::new(PathBuf::from("settings"), PathBuf::from("config")),
            StubVerificationStorage::default(),
        )
    }

    fn loader_at(
        location_info: LocationInfo,
        verification_storage: StubVerificationStorage,
    ) -> ExtismPluginLoader {
        ExtismPluginLoader::new(
            Arc::new(location_info),
            Arc::new(verification_storage),
            PluginSandboxLimits::default(),
        )
    }

    fn manifest_with_memory_limit(memory_limit: Option<usize>) -> PluginManifest {
        PluginManifest {
            metadata: PluginMetadata {
                id: "test".into(),
                name: "Test".into(),
                version: semver::Version::new(0, 1, 0),
                description: None,
                authors: vec![],
                license: None,
            },
            runtime: RuntimeConfig {
                allowed_hosts: vec![],
                allowed_paths: vec![],
            },
            load: LoadConfig::Extism {
                file: PathBuf::from("plugin.wasm"),
                memory_limit,
            },
            api: ApiConfig {
                version: semver::VersionReq::STAR,
                features: vec![],
                host_functions: Some(vec![]),
            },
        }
    }

    fn manifest_json(manifest: &PluginManifest) -> serde_json::Value {
        manifest_json_with_hash(manifest, None)
    }

    fn manifest_json_with_hash(
        manifest: &PluginManifest,
        expected_wasm_sha256: Option<String>,
    ) -> serde_json::Value {
        let wasm_manifest = loader().build_wasm_manifest(manifest, vec![], expected_wasm_sha256);
        serde_json::to_value(&wasm_manifest).expect("manifest should serialize")
    }

    /// Lays out `<dir>/plugins/test/plugin.wasm` with the given bytes and returns
    /// a loader rooted at `dir` plus the wasm path.
    fn loader_with_wasm(
        dir: &Path,
        wasm_bytes: &[u8],
        verification: Option<PluginVerification>,
    ) -> (ExtismPluginLoader, PathBuf) {
        let location_info = LocationInfo::new(dir.to_path_buf(), dir.to_path_buf());
        let wasm_path = location_info.plugin_dir("test").join("plugin.wasm");

        std::fs::create_dir_all(wasm_path.parent().expect("plugin dir"))
            .expect("plugin dir should be created");
        std::fs::write(&wasm_path, wasm_bytes).expect("wasm should be written");

        (
            loader_at(location_info, StubVerificationStorage(verification)),
            wasm_path,
        )
    }

    fn verification(wasm_sha256: &str) -> PluginVerification {
        PluginVerification::new(wasm_sha256, PluginSourceType::GitHub)
    }

    /// Stand-in for the blob a release shipped. Never reaches wasmtime: the
    /// integrity check runs — and here, fails — before the runtime is involved.
    const ORIGINAL_WASM: &[u8] = b"original wasm";

    #[test]
    fn should_round_bytes_up_to_pages_when_they_are_not_a_whole_number_of_pages() {
        assert_eq!(bytes_to_pages(0), 0);
        assert_eq!(bytes_to_pages(1), 1);
        assert_eq!(bytes_to_pages(WASM_PAGE_SIZE_BYTES), 1);
        assert_eq!(bytes_to_pages(WASM_PAGE_SIZE_BYTES + 1), 2);
        assert_eq!(bytes_to_pages(256 * 1024 * 1024), 4096);
    }

    #[test]
    fn should_saturate_pages_at_u32_max_when_the_size_is_huge() {
        assert_eq!(bytes_to_pages(usize::MAX), u32::MAX);
    }

    #[test]
    fn should_apply_memory_limit_and_timeout_when_the_manifest_sets_them() {
        let json = manifest_json(&manifest_with_memory_limit(Some(128 * 1024 * 1024)));

        assert_eq!(json["memory"]["max_pages"], 2048);
        assert_eq!(
            json["timeout_ms"],
            u64::try_from(DEFAULT_TIMEOUT.as_millis()).expect("the default timeout fits in u64")
        );
    }

    /// WASI may only read the instance tree: writes go through `write_instance_file`. The
    /// plugin cache stays read-write.
    #[test]
    fn should_mount_instances_read_only_and_the_cache_read_write_when_loading() {
        let manifest = manifest_with_memory_limit(None);
        let location_info = LocationInfo::new(PathBuf::from("settings"), PathBuf::from("config"));
        let defaults = get_default_allowed_paths(&location_info, &manifest.metadata.id);

        let allowed_paths = resolve_allowed_paths(&manifest, None, Some(&defaults), &[]);

        let host_for = |plugin: &str| {
            let Some(mapping) = allowed_paths
                .iter()
                .find(|mapping| mapping.1 == Path::new(plugin))
            else {
                panic!("`{plugin}` must be mounted");
            };

            mapping.0.clone()
        };

        assert_eq!(
            host_for("/instances"),
            format!("ro:{}", location_info.instances_dir().to_string_lossy())
        );
        assert_eq!(
            host_for("/cache"),
            location_info
                .plugin_cache_dir(&manifest.metadata.id)
                .to_string_lossy()
        );
    }

    /// A manifest's `allowedPaths` would be opened relative to the launcher's working directory,
    /// `..` included: none of them is mounted, whatever it names.
    #[test]
    fn should_mount_no_folder_when_the_manifest_declares_one() {
        let mut manifest = manifest_with_memory_limit(None);
        manifest.runtime.allowed_paths = vec![
            PathMapping("../..".into(), PathBuf::from("/launcher")),
            PathMapping("plugins".into(), PathBuf::from("/plugins")),
        ];
        assert!(
            manifest.runtime.validate().is_ok(),
            "still accepted, so that such a plugin keeps loading"
        );

        let allowed_paths = resolve_allowed_paths(&manifest, None, None, &[]);

        assert_eq!(allowed_paths, vec![]);
    }

    /// An absolute host path on this platform: a relative one is refused before anything else.
    fn absolute(name: &str) -> String {
        if cfg!(windows) {
            format!(r"C:\{name}")
        } else {
            format!("/{name}")
        }
    }

    /// A folder saved over a core mount must not displace it: it is left out, the rest stays.
    #[test]
    fn should_not_mount_a_saved_folder_when_it_lies_over_a_core_folder() {
        let manifest = manifest_with_memory_limit(None);
        let root = tempfile::tempdir().expect("temp dir");
        let location_info =
            LocationInfo::new(root.path().join("settings"), root.path().join("config"));
        let defaults = get_default_allowed_paths(&location_info, &manifest.metadata.id);
        let user_folder = |host: String, plugin: &str| AllowedPath {
            host,
            plugin: plugin.into(),
            read_only: false,
        };
        let settings = PluginSettings {
            allowed_paths: vec![
                user_folder(
                    location_info
                        .plugin_cache_dir(&manifest.metadata.id)
                        .to_string_lossy()
                        .into_owned(),
                    "/stolen",
                ),
                user_folder(
                    location_info
                        .instances_dir()
                        .join("a")
                        .to_string_lossy()
                        .into_owned(),
                    "/inside",
                ),
                // Where its own `settings.toml` and every other plugin's binary live.
                user_folder(
                    location_info.plugins_dir().to_string_lossy().into_owned(),
                    "/plugins",
                ),
                user_folder(
                    location_info
                        .plugin_dir(&manifest.metadata.id)
                        .to_string_lossy()
                        .into_owned(),
                    "/self",
                ),
                user_folder(absolute("packs"), "/mnt/d/packs"),
            ],
            ..PluginSettings::default()
        };
        let core = core_folders(&location_info, &manifest.metadata.id);

        let mounted = resolve_allowed_paths(&manifest, Some(&settings), Some(&defaults), &core);

        let plugin_paths: Vec<_> = mounted.iter().map(|m| m.1.clone()).collect();
        assert!(plugin_paths.contains(&PathBuf::from("/cache")));
        assert!(plugin_paths.contains(&PathBuf::from("/instances")));
        assert!(plugin_paths.contains(&PathBuf::from("/mnt/d/packs")));
        assert_eq!(mounted.len(), 3, "{plugin_paths:?}");
    }

    /// Saved folders are checked again on load: a relative host would be opened against the
    /// launcher's working directory, a plugin path over a core mount would replace it.
    #[test]
    fn should_not_mount_a_saved_folder_when_it_fails_validation() {
        let manifest = manifest_with_memory_limit(None);
        let root = tempfile::tempdir().expect("temp dir");
        let location_info =
            LocationInfo::new(root.path().join("settings"), root.path().join("config"));
        let defaults = get_default_allowed_paths(&location_info, &manifest.metadata.id);
        let user_folder = |host: String, plugin: &str| AllowedPath {
            host,
            plugin: plugin.into(),
            read_only: false,
        };
        let settings = PluginSettings {
            allowed_paths: vec![
                user_folder("packs".into(), "/packs"),
                user_folder("../..".into(), "/launcher"),
                user_folder(absolute("evil-cache"), "/cache"),
                user_folder(absolute("evil-instances"), "/instances/a"),
                user_folder(absolute("everything"), "/"),
                user_folder(absolute("unrooted"), "unrooted"),
                user_folder(absolute("dotted"), "/a/../instances"),
                user_folder(absolute(""), "/drive"),
                user_folder(absolute("kept"), "/kept"),
            ],
            ..PluginSettings::default()
        };
        let core = core_folders(&location_info, &manifest.metadata.id);

        let mounted = resolve_allowed_paths(&manifest, Some(&settings), Some(&defaults), &core);

        let mut user_mounts: Vec<_> = mounted
            .iter()
            .filter(|m| m.1 != Path::new("/cache") && m.1 != Path::new("/instances"))
            .collect();
        assert_eq!(
            user_mounts.pop(),
            Some(&PathMapping(absolute("kept"), PathBuf::from("/kept")))
        );
        assert!(user_mounts.is_empty(), "{user_mounts:?}");

        let core_hosts: Vec<_> = mounted
            .iter()
            .filter(|m| m.1 == Path::new("/cache") || m.1 == Path::new("/instances"))
            .map(|m| m.0.clone())
            .collect();
        assert_eq!(core_hosts.len(), 2, "the core mounts stay the core's");
        assert!(
            core_hosts.iter().all(|host| !host.contains("evil")),
            "{core_hosts:?}"
        );
    }

    /// The user's folders reach Extism read-only unless the user said otherwise.
    #[test]
    fn should_mount_the_user_folders_with_their_rights_when_settings_have_them() {
        let settings = PluginSettings {
            allowed_paths: vec![
                AllowedPath {
                    host: absolute("packs"),
                    plugin: "/mnt/d/packs".into(),
                    read_only: true,
                },
                AllowedPath {
                    host: absolute("out"),
                    plugin: "/mnt/d/out".into(),
                    read_only: false,
                },
            ],
            ..PluginSettings::default()
        };

        let allowed_paths = resolve_allowed_paths(
            &manifest_with_memory_limit(None),
            Some(&settings),
            None,
            &[],
        );

        assert_eq!(
            allowed_paths,
            vec![
                PathMapping(
                    format!("ro:{}", absolute("packs")),
                    PathBuf::from("/mnt/d/packs")
                ),
                PathMapping(absolute("out"), PathBuf::from("/mnt/d/out")),
            ]
        );
    }

    /// Hosts the plugin declares are the host functions' business: the Extism manifest never
    /// lists any, so the built-in `http_request` has nothing to allow.
    #[test]
    fn should_not_hand_allowed_hosts_to_extism_even_when_the_manifest_declares_some() {
        let mut manifest = manifest_with_memory_limit(None);
        manifest.runtime.allowed_hosts = vec!["github.com".into()];

        let json = manifest_json(&manifest);

        let hosts = json.get("allowed_hosts").filter(|hosts| !hosts.is_null());
        assert!(hosts.is_none(), "{json}");
    }

    #[test]
    fn should_merge_allowed_hosts_when_manifest_and_settings_both_have_them() {
        let mut manifest = manifest_with_memory_limit(None);
        manifest.runtime.allowed_hosts = vec!["github.com".into()];
        let settings = PluginSettings {
            allowed_hosts: vec!["*.example.com".into()],
            ..PluginSettings::default()
        };

        let hosts = crate::features::plugins::infra::plugin_utils::effective_allowed_hosts(
            &manifest,
            Some(&settings),
        );

        assert_eq!(hosts, vec!["github.com", "*.example.com"]);
    }

    #[test]
    fn should_apply_default_memory_limit_when_absent() {
        let json = manifest_json(&manifest_with_memory_limit(None));

        assert_eq!(
            json["memory"]["max_pages"],
            bytes_to_pages(PluginSandboxLimits::default().default_memory_limit_bytes)
        );
        assert_eq!(
            json["timeout_ms"],
            u64::try_from(DEFAULT_TIMEOUT.as_millis()).expect("the default timeout fits in u64")
        );
    }

    // ── wasm integrity ──

    #[tokio::test]
    async fn should_reject_wasm_when_it_was_tampered_with() {
        let dir = tempfile::tempdir().expect("temp dir");
        let recorded = sha256_async(ORIGINAL_WASM.to_vec()).await.unwrap();
        let (loader, wasm_path) =
            loader_with_wasm(dir.path(), b"tampered wasm", Some(verification(&recorded)));

        let error = loader
            .resolve_expected_wasm_sha256("test", &wasm_path)
            .await
            .expect_err("a tampered wasm file must not load");

        match error {
            PluginError::LoadFailed { plugin_id, reason } => {
                assert_eq!(plugin_id, "test");
                assert!(
                    reason.contains(&recorded),
                    "the error should name the recorded digest, got: {reason}"
                );
            }
            other => panic!("expected LoadFailed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn should_accept_wasm_when_it_matches_the_recorded_hash() {
        let dir = tempfile::tempdir().expect("temp dir");
        let recorded = sha256_async(ORIGINAL_WASM.to_vec()).await.unwrap();
        let (loader, wasm_path) =
            loader_with_wasm(dir.path(), ORIGINAL_WASM, Some(verification(&recorded)));

        let expected = loader
            .resolve_expected_wasm_sha256("test", &wasm_path)
            .await
            .expect("an untouched wasm file should pass");

        assert_eq!(expected, Some(recorded));
    }

    /// A plugin installed from a local archive has nothing to verify against: it loads,
    /// unverified, rather than being blocked.
    #[tokio::test]
    async fn should_load_an_unverified_plugin_when_no_hash_was_recorded() {
        let dir = tempfile::tempdir().expect("temp dir");
        let (loader, wasm_path) = loader_with_wasm(dir.path(), ORIGINAL_WASM, None);

        let expected = loader
            .resolve_expected_wasm_sha256("test", &wasm_path)
            .await
            .expect("an unverified plugin should still load");

        assert_eq!(expected, None);
    }

    /// The digest has to reach Extism too, so a file swapped between our check
    /// and its own read is still caught.
    #[test]
    fn should_pin_the_hash_in_the_extism_manifest_when_a_hash_was_recorded() {
        let json =
            manifest_json_with_hash(&manifest_with_memory_limit(None), Some("abc123".to_owned()));

        assert_eq!(json["wasm"][0]["hash"], "abc123");
    }

    #[test]
    fn should_leave_the_extism_manifest_unpinned_when_the_plugin_is_unverified() {
        let json = manifest_json(&manifest_with_memory_limit(None));

        assert!(json["wasm"][0]["hash"].is_null());
    }

    // ── host function declaration ──

    /// Stand-ins for the real host functions: same names and signatures, no container behind
    /// them. Linking looks at nothing else.
    fn stub_host_functions(declared: &[String]) -> Vec<Function> {
        select_host_functions(declared)
            .into_iter()
            .map(|spec| {
                Function::new(
                    spec.name,
                    spec.params.iter().cloned(),
                    spec.results.iter().cloned(),
                    UserData::new(()),
                    |_, _, _, _| Ok(()),
                )
            })
            .collect()
    }

    /// Builds a plugin from WAT source, linking only the host functions `declared` selects —
    /// the same path `load` takes, minus the container.
    fn build_declaring(wat: &str, declared: &[&str]) -> Result<Plugin, PluginError> {
        let declared: Vec<String> = declared.iter().map(|&name| name.to_owned()).collect();

        ExtismPluginLoader::build_plugin(
            "test",
            &Manifest::new([Wasm::data(wat.as_bytes().to_vec())]),
            None,
            stub_host_functions(&declared),
            PluginSandboxLimits::default().default_fuel_limit,
        )
    }

    /// Imports two host functions, with the signatures the core registers them under.
    const IMPORTS_LOG_AND_GET_ID: &str = r#"(module
        (import "extism:host/user" "log" (func (param i64 i64)))
        (import "extism:host/user" "get_id" (func (result i64))))"#;

    fn load_failure(result: Result<Plugin, PluginError>) -> String {
        match result {
            Err(PluginError::LoadFailed { reason, .. }) => reason,
            Err(other) => panic!("expected LoadFailed, got: {other:?}"),
            Ok(_) => panic!("the plugin must not load"),
        }
    }

    #[test]
    fn should_load_a_plugin_when_it_declares_every_host_function_it_imports() {
        build_declaring(IMPORTS_LOG_AND_GET_ID, &["log", "get_id"])
            .expect("every import is declared and linked");
    }

    /// Declaring more than the wasm imports costs nothing: the extra functions are linked and
    /// simply never called.
    #[test]
    fn should_load_a_plugin_when_it_declares_more_than_it_imports() {
        build_declaring(IMPORTS_LOG_AND_GET_ID, &["log", "get_id", "http_get"])
            .expect("unused declared functions must not block loading");
    }

    /// The narrowing is real: Extism links only what it is handed, and wasmtime refuses the
    /// module over the import nobody provided.
    #[test]
    fn should_refuse_a_plugin_when_it_imports_an_undeclared_host_function() {
        let reason = load_failure(build_declaring(IMPORTS_LOG_AND_GET_ID, &["log"]));

        assert!(
            reason.contains("`get_id`")
                && reason.contains("does not declare it in `api.hostFunctions`"),
            "the reason should name the undeclared import, got: {reason}"
        );
    }

    #[test]
    fn should_refuse_any_host_import_when_the_declaration_is_empty() {
        let reason = load_failure(build_declaring(IMPORTS_LOG_AND_GET_ID, &[]));

        assert!(reason.contains("api.hostFunctions"), "{reason}");
    }

    /// No declaration, no fallback: refused before the wasm is read, so the loader needs
    /// neither a wasm file nor a container here.
    #[tokio::test]
    async fn should_refuse_to_load_a_manifest_when_it_has_no_host_function_declaration() {
        let mut manifest = manifest_with_memory_limit(None);
        manifest.api.host_functions = None;

        let result = loader().load(&manifest, None).await;

        assert!(matches!(
            result,
            Err(PluginError::Manifest(
                ManifestError::HostFunctionsNotDeclared
            ))
        ));
    }

    #[test]
    fn should_name_a_host_function_when_the_core_does_not_have_it() {
        let reason = load_failure(build_declaring(
            r#"(module (import "extism:host/user" "launch_rockets" (func)))"#,
            &["log"],
        ));

        assert!(
            reason.contains("`launch_rockets`") && reason.contains("does not provide"),
            "{reason}"
        );
    }

    /// The signature has to match too — a name alone does not satisfy an import.
    #[test]
    fn should_pass_other_link_errors_through_when_the_import_is_not_a_missing_host_function() {
        let reason = load_failure(build_declaring(
            r#"(module (import "extism:host/user" "get_id" (func (param i32))))"#,
            &["get_id"],
        ));

        assert!(!reason.contains("api.hostFunctions"), "{reason}");
    }

    #[test]
    fn should_offer_every_registered_host_function_when_listing_declarable_ones() {
        assert_eq!(loader().host_functions(), host_function_names());
    }

    // ── `ro:` on real Extism ──

    /// Talks WASI directly to the first preopened folder (fd 3), the one mount these tests
    /// give it. Each export returns the WASI errno of the step that failed, 0 on success.
    const WASI_PLUGIN: &str = r#"(module
        (import "wasi_snapshot_preview1" "path_open"
            (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
        (import "wasi_snapshot_preview1" "fd_write"
            (func $fd_write (param i32 i32 i32 i32) (result i32)))
        (import "wasi_snapshot_preview1" "fd_read"
            (func $fd_read (param i32 i32 i32 i32) (result i32)))
        (import "wasi_snapshot_preview1" "fd_close" (func $fd_close (param i32) (result i32)))
        (memory (export "memory") 1)
        (data (i32.const 0) "pack.toml")
        (data (i32.const 16) "new.toml")
        (data (i32.const 32) "written")

        ;; fd lands at 80; iovec at 64; byte count at 84; read buffer at 128.
        (func $open (param $path i32) (param $len i32) (param $oflags i32) (param $rights i64)
            (result i32)
            (call $path_open (i32.const 3) (i32.const 0) (local.get $path) (local.get $len)
                (local.get $oflags) (local.get $rights) (i64.const 0) (i32.const 0)
                (i32.const 80)))

        ;; Opens pack.toml with FD_WRITE and writes "written" over its start.
        (func (export "write") (result i32)
            (local $rc i32)
            (local.set $rc (call $open (i32.const 0) (i32.const 9) (i32.const 0) (i64.const 64)))
            (if (local.get $rc) (then (return (local.get $rc))))
            (i32.store (i32.const 64) (i32.const 32))
            (i32.store (i32.const 68) (i32.const 7))
            (local.set $rc (call $fd_write (i32.load (i32.const 80)) (i32.const 64) (i32.const 1)
                (i32.const 84)))
            (drop (call $fd_close (i32.load (i32.const 80))))
            (local.get $rc))

        ;; Opens pack.toml with FD_READ and reads it; 200 if it is not the 8 bytes expected.
        (func (export "read") (result i32)
            (local $rc i32)
            (local.set $rc (call $open (i32.const 0) (i32.const 9) (i32.const 0) (i64.const 2)))
            (if (local.get $rc) (then (return (local.get $rc))))
            (i32.store (i32.const 64) (i32.const 128))
            (i32.store (i32.const 68) (i32.const 64))
            (local.set $rc (call $fd_read (i32.load (i32.const 80)) (i32.const 64) (i32.const 1)
                (i32.const 84)))
            (drop (call $fd_close (i32.load (i32.const 80))))
            (if (local.get $rc) (then (return (local.get $rc))))
            (if (i32.ne (i32.load (i32.const 84)) (i32.const 8)) (then (return (i32.const 200))))
            (i32.const 0))

        ;; O_CREAT with FD_READ only: no write right asked for.
        (func (export "create") (result i32)
            (local $rc i32)
            (local.set $rc (call $open (i32.const 16) (i32.const 8) (i32.const 1) (i64.const 2)))
            (if (local.get $rc) (then (return (local.get $rc))))
            (drop (call $fd_close (i32.load (i32.const 80))))
            (i32.const 0))

        ;; O_CREAT | O_TRUNC on pack.toml with FD_READ only.
        (func (export "truncate") (result i32)
            (local $rc i32)
            (local.set $rc (call $open (i32.const 0) (i32.const 9) (i32.const 9) (i64.const 2)))
            (if (local.get $rc) (then (return (local.get $rc))))
            (drop (call $fd_close (i32.load (i32.const 80))))
            (i32.const 0)))"#;

    const PACK: &[u8] = b"original";

    /// A folder holding `pack.toml`, mounted the way `load` mounts a folder of the user's.
    fn sandbox(read_only: bool) -> (tempfile::TempDir, Plugin) {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("pack.toml"), PACK).expect("pack.toml");

        let settings = PluginSettings {
            allowed_paths: vec![AllowedPath {
                host: dir.path().to_string_lossy().into_owned(),
                plugin: "/pack".into(),
                read_only,
            }],
            ..PluginSettings::default()
        };
        let allowed_paths = resolve_allowed_paths(
            &manifest_with_memory_limit(None),
            Some(&settings),
            None,
            &[],
        );

        let plugin = ExtismPluginLoader::build_plugin(
            "test",
            &Manifest::new([Wasm::data(WASI_PLUGIN.as_bytes().to_vec())])
                .with_allowed_paths(allowed_paths.into_iter().map(Into::into)),
            None,
            vec![],
            PluginSandboxLimits::default().default_fuel_limit,
        )
        .expect("the WASI plugin should load");

        (dir, plugin)
    }

    /// The export's return value. `call_get_error_code` does not give it back (it drops a
    /// non-zero code that comes without an Extism error); `call` reports it only in its message.
    fn errno(plugin: &mut Plugin, function: &str) -> i32 {
        const NON_ZERO: &str = "Returned non-zero exit code: ";

        match plugin.call::<&[u8], &[u8]>(function, &[]) {
            Ok(_) => 0,
            Err(err) => {
                let message = err.to_string();
                message
                    .strip_prefix(NON_ZERO)
                    .and_then(|code| code.parse().ok())
                    .unwrap_or_else(|| panic!("`{function}` failed outside WASI: {message}"))
            }
        }
    }

    fn pack(dir: &tempfile::TempDir) -> Vec<u8> {
        std::fs::read(dir.path().join("pack.toml")).expect("pack.toml")
    }

    /// The control: the same calls do write through a read-write mount, so a refusal below is
    /// the mount's doing and not a broken test plugin.
    #[test]
    fn should_write_through_a_mount_when_it_is_read_write() {
        let (dir, mut plugin) = sandbox(false);

        assert_eq!(errno(&mut plugin, "write"), 0);
        assert_eq!(pack(&dir), b"writtenl");
    }

    #[test]
    fn should_read_but_not_write_through_a_mount_when_it_is_read_only() {
        let (dir, mut plugin) = sandbox(true);

        assert_ne!(
            errno(&mut plugin, "write"),
            0,
            "opening for write is refused"
        );
        assert_eq!(pack(&dir), PACK, "the file is untouched");
        assert_eq!(errno(&mut plugin, "read"), 0, "reading the same file works");
    }

    /// Extism's read-only folder refuses only opens asking for the write right, but WASI turns
    /// `O_CREAT` into a write open of its own: a plugin can create or empty files in a read-only
    /// mount. Pinned so that a change in Extism either way is noticed.
    #[test]
    fn should_let_o_creat_through_when_the_mount_is_read_only() {
        let (dir, mut plugin) = sandbox(true);

        let created = errno(&mut plugin, "create");
        let truncated = errno(&mut plugin, "truncate");

        assert_eq!(
            (created, dir.path().join("new.toml").exists()),
            (0, true),
            "O_CREAT without the write right creates the file"
        );
        assert_eq!(
            (truncated, pack(&dir)),
            (0, Vec::new()),
            "O_CREAT | O_TRUNC without the write right empties the file"
        );
    }

    /// The empty wasm module: magic + version, nothing else. Valid enough for
    /// wasmtime to compile, which is all these two tests need.
    const EMPTY_WASM_MODULE: &[u8] = b"\0asm\x01\0\0\0";

    /// Writes a wasmtime cache config so the builder below runs with the cache
    /// enabled, exactly as `load` does.
    fn cache_config_file(dir: &Path) -> PathBuf {
        let cache_dir = dir.join("cache");
        let cache_config_path = dir.join("cache.toml");

        std::fs::create_dir_all(&cache_dir).expect("cache dir should be created");
        std::fs::write(
            &cache_config_path,
            toml::to_string(&get_default_cache_config(cache_dir)).expect("cache config"),
        )
        .expect("cache config should be written");

        cache_config_path
    }

    fn build_with_hash(dir: &Path, hash: &str) -> Result<Plugin, extism::Error> {
        let wasm_path = dir.join("plugin.wasm");
        std::fs::write(&wasm_path, EMPTY_WASM_MODULE).expect("wasm should be written");

        let wasm_manifest = Manifest::new([Wasm::file(wasm_path).with_hash(hash)]);

        PluginBuilder::new(&wasm_manifest)
            .with_wasi(true)
            .with_cache_config(cache_config_file(dir))
            .build()
    }

    /// The wasmtime compilation cache must not let a swapped file through: Extism hashes the
    /// bytes it reads before wasmtime sees them.
    #[tokio::test]
    async fn should_reject_a_bad_hash_when_the_compilation_cache_is_enabled() {
        let dir = tempfile::tempdir().expect("temp dir");

        // Prime the cache with the very same module under its real digest…
        let real_hash = sha256_async(EMPTY_WASM_MODULE.to_vec()).await.unwrap();
        build_with_hash(dir.path(), &real_hash).expect("the untouched module should load");

        // …then ask for it under a digest that no longer matches.
        let error = build_with_hash(dir.path(), &"0".repeat(64))
            .expect_err("a cached module must not bypass the hash check");

        assert!(
            error.to_string().contains("Hash mismatch"),
            "expected a hash mismatch, got: {error}"
        );
    }
}
