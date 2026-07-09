use std::sync::Arc;

use crate::features::plugins::app::PluginProviderFactory;
use crate::features::plugins::app::ports::{
    CheckForPluginUpdatesUseCasePort, EditPluginSettingsUseCasePort, EnablePluginUseCasePort,
    ForceEnablePluginUseCasePort, GetPluginApiVersionUseCasePort, GetPluginDtoUseCasePort,
    GetPluginSettingsUseCasePort, ImportPluginsUseCasePort, ListPluginsDtoUseCasePort,
    PluginDisableService, PluginExtractor, PluginLoader, PluginSettingsStorage,
    PluginSourceStorage, PluginStorage, PluginSyncService, RemovePluginUseCasePort,
    UpdatePluginUseCasePort,
};
use crate::features::plugins::app::services::{PluginLoaderRegistry, PluginRegistry};

/// Extension trait providing access to all plugins feature use cases and services.
///
/// Implemented on the core dependency injection container to expose
/// plugins-specific functionality in a centralized manner.
pub trait PluginsFeature {
    // ── Use cases (via port traits) ──
    fn check_for_plugin_updates_use_case(&self) -> Arc<dyn CheckForPluginUpdatesUseCasePort>;
    fn edit_plugin_settings_use_case(&self) -> Arc<dyn EditPluginSettingsUseCasePort>;
    fn enable_plugin_use_case(&self) -> Arc<dyn EnablePluginUseCasePort>;
    fn force_enable_plugin_use_case(&self) -> Arc<dyn ForceEnablePluginUseCasePort>;
    fn get_plugin_api_version_use_case(&self) -> Arc<dyn GetPluginApiVersionUseCasePort>;
    fn get_plugin_dto_use_case(&self) -> Arc<dyn GetPluginDtoUseCasePort>;
    fn get_plugin_settings_use_case(&self) -> Arc<dyn GetPluginSettingsUseCasePort>;
    fn import_plugins_use_case(&self) -> Arc<dyn ImportPluginsUseCasePort>;
    fn list_plugins_dto_use_case(&self) -> Arc<dyn ListPluginsDtoUseCasePort>;
    fn remove_plugin_use_case(&self) -> Arc<dyn RemovePluginUseCasePort>;
    fn update_plugin_use_case(&self) -> Arc<dyn UpdatePluginUseCasePort>;

    // ── Use cases (via service trait ports) ──
    fn disable_plugin_use_case(&self) -> Arc<dyn PluginDisableService>;
    fn sync_plugins_use_case(&self) -> Arc<dyn PluginSyncService>;

    // ── Services ──
    fn plugin_registry(&self) -> Arc<PluginRegistry>;
    fn plugin_loader_registry(&self) -> Arc<PluginLoaderRegistry>;
    fn plugin_provider_factory(&self) -> Arc<PluginProviderFactory>;

    // ── Ports ──
    fn plugin_source_storage(&self) -> Arc<dyn PluginSourceStorage>;
    fn plugin_loader(&self) -> Arc<dyn PluginLoader>;
    fn plugin_storage(&self) -> Arc<dyn PluginStorage>;
    fn plugin_settings_storage(&self) -> Arc<dyn PluginSettingsStorage>;
    fn plugin_extractor(&self) -> Arc<dyn PluginExtractor>;
}
