use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use extism::{Manifest, Plugin, PluginBuilder, Wasm};
use tokio::sync::Mutex;

use crate::{
    features::{
        plugins::{
            LoadConfig, PathMapping, PluginError, PluginInstance, PluginLoader, PluginManifest,
            PluginSettings,
        },
        settings::LocationInfo,
    },
    shared::{create_dir_all, write_toml_async},
};

use super::{
    host_functions::get_host_functions,
    models::{ExtismPluginInstance, WasmCache, WasmCacheConfig},
};

use super::super::plugin_utils::get_default_allowed_paths;

pub struct ExtismPluginLoader {
    location_info: Arc<LocationInfo>,
}

impl ExtismPluginLoader {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
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
        let wasm_file_path = match &manifest.load {
            LoadConfig::Extism { file, .. } => file,
            config => {
                return Err(PluginError::InvalidConfig {
                    config: config.clone(),
                })
            }
        };

        let wasm_file =
            Wasm::file(self.resolve_absolute_wasm_path(&manifest.metadata.id, wasm_file_path));

        let (allowed_hosts, allowed_paths) =
            resolve_allowed_paths(manifest, settings, default_allowed_paths);

        Ok(Manifest::new([wasm_file])
            .with_allowed_hosts(allowed_hosts.into_iter())
            .with_allowed_paths(allowed_paths.into_iter().map(Into::into)))
    }

    fn build_plugin(
        &self,
        plugin_id: &str,
        wasm_manifest: &Manifest,
        cache_dir: Option<&PathBuf>,
    ) -> Result<Plugin, PluginError> {
        let mut builder = PluginBuilder::new(wasm_manifest)
            .with_functions(get_host_functions(plugin_id))
            .with_wasi(true);

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

        let wasm_manifest =
            self.build_wasm_manifest(manifest, Some(&default_allowed_paths), settings)?;

        let extism_plugin = self.build_plugin(plugin_id, &wasm_manifest, Some(&cache_config))?;

        let mut plugin = ExtismPluginInstance::new(extism_plugin, plugin_id.clone());
        if let Err(err) = plugin.on_load() {
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

        if let Err(err) = plugin.on_unload() {
            tracing::debug!(
                "Failed to call on_unload on plugin {}: {:?}",
                plugin.get_id(),
                err
            );
        }

        Ok(())
    }
}

fn get_default_cache_config(cache_dir: PathBuf) -> WasmCacheConfig {
    WasmCacheConfig {
        cache: WasmCache {
            enabled: true,
            cleanup_interval: "30m".to_owned(),
            files_total_size_soft_limit: "1Gi".to_owned(),
            directory: cache_dir,
        },
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
