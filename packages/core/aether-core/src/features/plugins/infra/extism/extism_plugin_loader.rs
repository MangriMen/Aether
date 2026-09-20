use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, Weak},
};

use async_trait::async_trait;
use extism::{Manifest, Plugin, PluginBuilder, Wasm};
use tokio::sync::Mutex;

use crate::{
    core::app::AetherContainer,
    features::{
        plugins::{
            LoadConfig, PathMapping, PluginError, PluginInstance, PluginInternalEvent,
            PluginLoader, PluginManifest, PluginSettings, PluginVerificationStorage,
        },
        settings::LocationInfo,
    },
    shared::{
        hash::infra::sha256_async,
        io::infra::{create_dir_all, read_async, write_toml_async},
    },
};

use super::{
    extism_limits::{
        DEFAULT_FUEL_LIMIT, DEFAULT_MEMORY_LIMIT_BYTES, DEFAULT_TIMEOUT, bytes_to_pages,
    },
    host_functions::get_host_functions,
    models::{ExtismPluginInstance, get_default_cache_config},
};

use super::super::plugin_utils::get_default_allowed_paths;

pub struct ExtismPluginLoader {
    location_info: Arc<LocationInfo>,
    verification_storage: Arc<dyn PluginVerificationStorage>,
    container: OnceLock<Weak<AetherContainer>>,
}

impl ExtismPluginLoader {
    pub fn new(
        location_info: Arc<LocationInfo>,
        verification_storage: Arc<dyn PluginVerificationStorage>,
    ) -> Self {
        Self {
            location_info,
            verification_storage,
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
    fn extism_load_config(
        &self,
        manifest: &PluginManifest,
    ) -> Result<(PathBuf, usize), PluginError> {
        match &manifest.load {
            LoadConfig::Extism { file, memory_limit } => Ok((
                self.resolve_absolute_wasm_path(&manifest.metadata.id, file),
                memory_limit.unwrap_or(DEFAULT_MEMORY_LIMIT_BYTES),
            )),
            config @ LoadConfig::Native { .. } => Err(PluginError::InvalidConfig {
                config: config.clone(),
            }),
        }
    }

    /// Matches the wasm blob on disk against the TOFU record taken when the
    /// plugin was downloaded (`OPEN_QUESTIONS.md`, Q4).
    ///
    /// Returns the digest to pin in the Extism manifest, or `None` for a plugin
    /// that has no record — one installed from a local archive rather than
    /// through a provider. Those still load, but unverified and loudly so.
    ///
    /// Checking here as well as through `Wasm::with_hash` is deliberate: this
    /// pass produces an error the user can act on, while `with_hash` closes the
    /// gap between this read and Extism's own.
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

        let actual_sha256 = sha256_async(read_async(wasm_path).await?).await;

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
        allowed_hosts: Vec<String>,
        allowed_paths: Vec<PathMapping>,
        expected_wasm_sha256: Option<String>,
    ) -> Result<Manifest, PluginError> {
        let (wasm_file_path, memory_limit_bytes) = self.extism_load_config(manifest)?;

        let mut wasm_file = Wasm::file(wasm_file_path);
        if let Some(sha256) = expected_wasm_sha256 {
            // `with_hash` belongs to `Wasm`, not to `Manifest` or `PluginBuilder`.
            // Extism checks it on the bytes it reads from disk, before handing
            // them to wasmtime, so the compilation cache cannot serve a module
            // for a file that no longer matches
            // (`extism-1.30.0/src/manifest.rs`, `to_module` → `check_hash`).
            wasm_file = wasm_file.with_hash(sha256);
        }

        Ok(Manifest::new([wasm_file])
            .with_allowed_hosts(allowed_hosts.into_iter())
            .with_allowed_paths(allowed_paths.into_iter().map(Into::into))
            .with_memory_max(bytes_to_pages(memory_limit_bytes))
            .with_timeout(DEFAULT_TIMEOUT))
    }

    fn build_plugin(
        plugin_id: &str,
        wasm_manifest: &Manifest,
        cache_dir: Option<&PathBuf>,
        container: &Arc<AetherContainer>,
        allowed_hosts: Vec<String>,
    ) -> Result<Plugin, PluginError> {
        let mut builder = PluginBuilder::new(wasm_manifest)
            .with_functions(get_host_functions(plugin_id, container, allowed_hosts))
            .with_wasi(true)
            .with_fuel_limit(DEFAULT_FUEL_LIMIT);

        if let Some(cache_dir) = cache_dir {
            builder = builder.with_cache_config(cache_dir);
        }

        builder.build().map_err(|e| {
            let err = PluginError::LoadFailed {
                plugin_id: plugin_id.to_owned(),
                reason: e.to_string(),
            };
            tracing::debug!("Load failed for plugin {}: {}", plugin_id, e);
            err
        })
    }
}

#[async_trait]
impl PluginLoader for ExtismPluginLoader {
    async fn load(
        &self,
        manifest: &PluginManifest,
        settings: Option<&PluginSettings>,
    ) -> Result<Arc<Mutex<dyn PluginInstance>>, PluginError> {
        let plugin_id = &manifest.metadata.id;

        // Before anything else touches the file: a plugin whose binary no
        // longer matches what was installed must not reach the runtime at all.
        let (wasm_path, _) = self.extism_load_config(manifest)?;
        let expected_wasm_sha256 = self
            .resolve_expected_wasm_sha256(plugin_id, &wasm_path)
            .await?;

        let cache_config = self.ensure_cache_config_file().await?;
        let default_allowed_paths = self.ensure_default_allowed_paths(plugin_id).await?;

        let container = self
            .container
            .get()
            .and_then(Weak::upgrade)
            .expect("ExtismPluginLoader::set_container must be called before loading plugins");

        // Resolved once and shared: the Extism manifest and the host functions must agree on
        // exactly which hosts the plugin may reach (see `features::http`).
        let (allowed_hosts, allowed_paths) =
            resolve_allowed_paths(manifest, settings, Some(&default_allowed_paths));

        let wasm_manifest = self.build_wasm_manifest(
            manifest,
            allowed_hosts.clone(),
            allowed_paths,
            expected_wasm_sha256,
        )?;

        let extism_plugin = Self::build_plugin(
            plugin_id,
            &wasm_manifest,
            Some(&cache_config),
            &container,
            allowed_hosts,
        )?;

        let mut plugin = ExtismPluginInstance::new(extism_plugin, plugin_id.clone());
        if let Err(err) = plugin.handle_event(&PluginInternalEvent::Loaded) {
            tracing::debug!(
                "Failed to call on_load on plugin {}: {:?}",
                plugin.get_id(),
                err
            );
        }

        Ok(Arc::new(Mutex::new(plugin)))
    }

    async fn unload(&self, instance: Arc<Mutex<dyn PluginInstance>>) -> Result<(), PluginError> {
        let mut plugin = instance.lock().await;

        if let Err(err) = plugin.handle_event(&PluginInternalEvent::Unloaded) {
            tracing::debug!(
                "Failed to call on_unload on plugin {}: {:?}",
                plugin.get_id(),
                err
            );
        }

        Ok(())
    }
}

fn resolve_allowed_paths(
    manifest: &PluginManifest,
    settings: Option<&PluginSettings>,
    default_allowed_paths: Option<&HashMap<String, PathBuf>>,
) -> (Vec<String>, Vec<PathMapping>) {
    let mut allowed_hosts = manifest.runtime.allowed_hosts.clone();
    let mut allowed_paths = manifest.runtime.allowed_paths.clone();

    if let Some(default_allowed_paths) = default_allowed_paths {
        allowed_paths.extend(
            default_allowed_paths
                .iter()
                .map(|(k, v)| PathMapping(k.clone(), v.clone())),
        );
    }

    if let Some(settings) = settings {
        allowed_hosts.extend_from_slice(&settings.allowed_hosts);
        allowed_paths.extend_from_slice(&settings.allowed_paths);
    }

    (allowed_hosts, allowed_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::plugins::domain::{ApiConfig, PluginMetadata, RuntimeConfig};

    use crate::features::plugins::{PluginSourceType, PluginVerification};

    use super::super::extism_limits::WASM_PAGE_SIZE_BYTES;

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
        ExtismPluginLoader::new(Arc::new(location_info), Arc::new(verification_storage))
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
        let wasm_manifest = loader()
            .build_wasm_manifest(manifest, vec![], vec![], expected_wasm_sha256)
            .expect("manifest should build");
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
    fn should_round_bytes_up_to_pages() {
        assert_eq!(bytes_to_pages(0), 0);
        assert_eq!(bytes_to_pages(1), 1);
        assert_eq!(bytes_to_pages(WASM_PAGE_SIZE_BYTES), 1);
        assert_eq!(bytes_to_pages(WASM_PAGE_SIZE_BYTES + 1), 2);
        assert_eq!(bytes_to_pages(256 * 1024 * 1024), 4096);
    }

    #[test]
    fn should_saturate_pages_at_u32_max() {
        assert_eq!(bytes_to_pages(usize::MAX), u32::MAX);
    }

    #[test]
    fn should_apply_memory_limit_and_timeout_from_manifest() {
        let json = manifest_json(&manifest_with_memory_limit(Some(128 * 1024 * 1024)));

        assert_eq!(json["memory"]["max_pages"], 2048);
        assert_eq!(json["timeout_ms"], DEFAULT_TIMEOUT.as_millis() as u64);
    }

    /// The Extism manifest and the `http_get` host function are handed the *same* resolved
    /// list, so a host reachable through one is reachable through the other — no more, no less.
    #[test]
    fn should_merge_allowed_hosts_from_manifest_and_settings() {
        let mut manifest = manifest_with_memory_limit(None);
        manifest.runtime.allowed_hosts = vec!["github.com".into()];

        let settings = PluginSettings {
            allowed_hosts: vec!["*.example.com".into()],
            ..PluginSettings::default()
        };

        let (allowed_hosts, _) = resolve_allowed_paths(&manifest, Some(&settings), None);

        assert_eq!(allowed_hosts, vec!["github.com", "*.example.com"]);
    }

    #[test]
    fn should_keep_allowed_hosts_empty_without_manifest_or_settings_entries() {
        let manifest = manifest_with_memory_limit(None);

        let (allowed_hosts, _) = resolve_allowed_paths(&manifest, None, None);

        assert!(allowed_hosts.is_empty());
    }

    #[test]
    fn should_apply_default_memory_limit_when_absent() {
        let json = manifest_json(&manifest_with_memory_limit(None));

        assert_eq!(
            json["memory"]["max_pages"],
            bytes_to_pages(DEFAULT_MEMORY_LIMIT_BYTES)
        );
        assert_eq!(json["timeout_ms"], DEFAULT_TIMEOUT.as_millis() as u64);
    }

    // ── wasm integrity (T-0.2) ──

    #[tokio::test]
    async fn should_reject_tampered_wasm() {
        let dir = tempfile::tempdir().expect("temp dir");
        let recorded = sha256_async(ORIGINAL_WASM.to_vec()).await;
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
    async fn should_accept_wasm_matching_the_recorded_hash() {
        let dir = tempfile::tempdir().expect("temp dir");
        let recorded = sha256_async(ORIGINAL_WASM.to_vec()).await;
        let (loader, wasm_path) =
            loader_with_wasm(dir.path(), ORIGINAL_WASM, Some(verification(&recorded)));

        let expected = loader
            .resolve_expected_wasm_sha256("test", &wasm_path)
            .await
            .expect("an untouched wasm file should pass");

        assert_eq!(expected, Some(recorded));
    }

    /// Q4: a plugin installed from a local archive has nothing to verify
    /// against. It loads, unverified, rather than being blocked.
    #[tokio::test]
    async fn should_load_unverified_plugin_without_a_recorded_hash() {
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
    fn should_pin_the_hash_in_the_extism_manifest() {
        let json =
            manifest_json_with_hash(&manifest_with_memory_limit(None), Some("abc123".to_owned()));

        assert_eq!(json["wasm"][0]["hash"], "abc123");
    }

    #[test]
    fn should_leave_the_extism_manifest_unpinned_for_unverified_plugins() {
        let json = manifest_json(&manifest_with_memory_limit(None));

        assert!(json["wasm"][0]["hash"].is_null());
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

    /// Trap from the task card: the wasmtime compilation cache must not let a
    /// swapped file through. It cannot — Extism hashes the bytes it reads from
    /// disk before wasmtime ever sees them.
    #[tokio::test]
    async fn should_reject_a_bad_hash_even_with_the_compilation_cache_enabled() {
        let dir = tempfile::tempdir().expect("temp dir");

        // Prime the cache with the very same module under its real digest…
        let real_hash = sha256_async(EMPTY_WASM_MODULE.to_vec()).await;
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
