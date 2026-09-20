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
            PluginLoader, PluginManifest, PluginSettings,
        },
        settings::LocationInfo,
    },
    shared::io::infra::{create_dir_all, write_toml_async},
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
    container: OnceLock<Weak<AetherContainer>>,
}

impl ExtismPluginLoader {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self {
            location_info,
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

    fn build_wasm_manifest(
        &self,
        manifest: &PluginManifest,
        default_allowed_paths: Option<&HashMap<String, PathBuf>>,
        settings: Option<&PluginSettings>,
    ) -> Result<Manifest, PluginError> {
        let (wasm_file_path, memory_limit_bytes) = match &manifest.load {
            LoadConfig::Extism { file, memory_limit } => {
                (file, memory_limit.unwrap_or(DEFAULT_MEMORY_LIMIT_BYTES))
            }
            config @ LoadConfig::Native { .. } => {
                return Err(PluginError::InvalidConfig {
                    config: config.clone(),
                });
            }
        };

        let wasm_file =
            Wasm::file(self.resolve_absolute_wasm_path(&manifest.metadata.id, wasm_file_path));

        let (allowed_hosts, allowed_paths) =
            resolve_allowed_paths(manifest, settings, default_allowed_paths);

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
    ) -> Result<Plugin, PluginError> {
        let mut builder = PluginBuilder::new(wasm_manifest)
            .with_functions(get_host_functions(plugin_id, container))
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

        let cache_config = self.ensure_cache_config_file().await?;
        let default_allowed_paths = self.ensure_default_allowed_paths(plugin_id).await?;

        let container = self
            .container
            .get()
            .and_then(Weak::upgrade)
            .expect("ExtismPluginLoader::set_container must be called before loading plugins");

        let wasm_manifest =
            self.build_wasm_manifest(manifest, Some(&default_allowed_paths), settings)?;

        let extism_plugin =
            Self::build_plugin(plugin_id, &wasm_manifest, Some(&cache_config), &container)?;

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

    use super::super::extism_limits::WASM_PAGE_SIZE_BYTES;

    fn loader() -> ExtismPluginLoader {
        ExtismPluginLoader::new(Arc::new(LocationInfo::new(
            PathBuf::from("settings"),
            PathBuf::from("config"),
        )))
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
        let wasm_manifest = loader()
            .build_wasm_manifest(manifest, None, None)
            .expect("manifest should build");
        serde_json::to_value(&wasm_manifest).expect("manifest should serialize")
    }

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

    #[test]
    fn should_apply_default_memory_limit_when_absent() {
        let json = manifest_json(&manifest_with_memory_limit(None));

        assert_eq!(
            json["memory"]["max_pages"],
            bytes_to_pages(DEFAULT_MEMORY_LIMIT_BYTES)
        );
        assert_eq!(json["timeout_ms"], DEFAULT_TIMEOUT.as_millis() as u64);
    }
}
