use std::{path::Path, sync::Arc};

use daedalus::get_path_from_artifact;

use crate::features::{
    java::JavaQueryService,
    minecraft::{
        LoaderVersionPreference, LoaderVersionService, MinecraftApplicationError,
        MinecraftDownloader, ModLoader, VersionManifestService, get_compatible_java_version,
        parse_rules, resolve_minecraft_version, vanilla,
    },
    settings::LocationInfo,
};

#[derive(Debug, Clone)]
pub struct MinecraftHealthParams {
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub java_path: Option<String>,
}

#[allow(dead_code)]
pub struct MinecraftHealthService {
    loader_version_resolver: Arc<dyn LoaderVersionService>,
    get_version_manifest_use_case: Arc<dyn VersionManifestService>,
    minecraft_downloader: Arc<dyn MinecraftDownloader>,
    get_java_use_case: Arc<dyn JavaQueryService>,
    location_info: Arc<LocationInfo>,
}

impl MinecraftHealthService {
    pub fn new(
        loader_version_resolver: Arc<dyn LoaderVersionService>,
        get_version_manifest_use_case: Arc<dyn VersionManifestService>,
        minecraft_downloader: Arc<dyn MinecraftDownloader>,
        get_java_use_case: Arc<dyn JavaQueryService>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            loader_version_resolver,
            get_version_manifest_use_case,
            minecraft_downloader,
            get_java_use_case,
            location_info,
        }
    }

    /// Perform a stat-only health check of all critical Minecraft files.
    ///
    /// Returns:
    /// - `Ok(true)` — all files present, launch can proceed without install
    /// - `Ok(false)` — some files missing, full install should run
    /// - `Err` — network/metadata error (offline), caller should skip check
    pub async fn verify_files(
        &self,
        params: MinecraftHealthParams,
    ) -> Result<bool, MinecraftApplicationError> {
        let MinecraftHealthParams {
            game_version,
            loader,
            loader_version,
            java_path,
        } = params;

        // --- Step 1: Resolve version manifest (from cache) ---
        let version_manifest = self.get_version_manifest_use_case.execute().await?;
        let (version, minecraft_updated) =
            resolve_minecraft_version(&game_version, &version_manifest)?;

        // --- Step 2: Resolve loader version (from cache) ---
        let loader_version = self
            .loader_version_resolver
            .resolve(&game_version, &loader, loader_version.as_ref())
            .await?;

        let version_jar = loader_version.as_ref().map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

        // --- Step 3: Fetch version info (from cache or network) ---
        // If offline, return Ok(true) to skip check rather than crash
        let Ok(version_info) = self
            .minecraft_downloader
            .get_version_info(&version, loader_version.as_ref(), None, None)
            .await
        else {
            return Ok(true);
        };

        // --- Step 4: Check Java binary ---
        let java_ok = if let Some(java_path) = java_path.as_ref() {
            Path::new(java_path).exists()
        } else {
            let compatible_version = get_compatible_java_version(&version_info);
            match self.get_java_use_case.execute(compatible_version).await {
                Ok(java) => Path::new(java.path()).exists(),
                Err(_) => false,
            }
        };

        if !java_ok {
            return Ok(false);
        }

        // --- Step 5: Check client.jar ---
        let client_path = self
            .location_info
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        if !client_path.exists() {
            return Ok(false);
        }

        // --- Step 6: Check every library (with rules filtering) ---
        let libraries_dir = self.location_info.libraries_dir();
        let java_arch = "unknown"; // conservative — matches most rules

        for library in &version_info.libraries {
            if !should_check_library(library, java_arch, minecraft_updated) {
                continue;
            }

            let Ok(lib_path_part) = get_path_from_artifact(&library.name) else {
                continue;
            };

            let lib_path = libraries_dir.join(&lib_path_part);
            if !lib_path.exists() {
                return Ok(false);
            }
        }

        // --- Step 7: Check asset index exists ---
        let index_path = self
            .location_info
            .assets_index_dir()
            .join(format!("{}.json", version_info.asset_index.id));

        if !index_path.exists() {
            return Ok(false);
        }

        Ok(true)
    }
}

/// Mirrors the filtering logic from `LibrariesService::should_download_library`
/// to avoid checking irrelevant libraries (e.g. native-only on wrong OS).
fn should_check_library(
    library: &vanilla::Library,
    java_arch: &str,
    minecraft_updated: bool,
) -> bool {
    if let Some(rules) = &library.rules
        && !parse_rules(rules, java_arch, minecraft_updated)
    {
        return false;
    }

    if !library.downloadable {
        return false;
    }

    true
}
