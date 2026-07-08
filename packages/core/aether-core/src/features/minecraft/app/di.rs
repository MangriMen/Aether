use std::sync::Arc;

use crate::features::minecraft::app::ports::{
    GetLoaderVersionManifestUseCasePort, LoaderVersionService, MetadataStorage,
    MinecraftDownloader, MinecraftHealthService, MinecraftInstallService,
    MinecraftLaunchCommandService, ModLoaderProcessor, VersionManifestService,
};

/// Extension trait providing access to all Minecraft feature use cases and services.
///
/// Implemented on the core dependency injection container to expose
/// Minecraft-specific functionality in a centralized manner.
pub trait MinecraftFeature {
    // ── Use cases (via service trait ports) ──
    fn get_version_manifest_use_case(&self) -> Arc<dyn VersionManifestService>;
    fn install_minecraft_use_case(&self) -> Arc<dyn MinecraftInstallService>;
    fn get_minecraft_launch_command_use_case(&self) -> Arc<dyn MinecraftLaunchCommandService>;

    // ── Use cases (via dedicated port) ──
    fn get_loader_version_manifest_use_case(&self) -> Arc<dyn GetLoaderVersionManifestUseCasePort>;

    // ── Ports ──
    fn loader_version_service(&self) -> Arc<dyn LoaderVersionService>;
    fn metadata_storage(&self) -> Arc<dyn MetadataStorage>;
    fn minecraft_downloader(&self) -> Arc<dyn MinecraftDownloader>;
    fn minecraft_health_service(&self) -> Arc<dyn MinecraftHealthService>;
    fn mod_loader_processor(&self) -> Arc<dyn ModLoaderProcessor>;
}
