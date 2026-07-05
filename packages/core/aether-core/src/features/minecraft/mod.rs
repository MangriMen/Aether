pub(crate) mod acl;
mod app;
mod domain;
pub mod infra;

// Re-export domain types explicitly (domain is private, but pub use works from private modules)
pub use acl::models::modded;
pub use acl::models::vanilla;
pub use acl::services::parse_arguments;
pub use acl::services::parse_rules;
pub use domain::LaunchSettings;
pub use domain::LoaderVersionPreference;
pub use domain::MinecraftDomainError;
pub use domain::ModLoader;
pub use domain::TEMPORARY_REPLACE_CHAR;
pub use domain::get_compatible_java_version;
pub use domain::resolve_minecraft_version;

// Re-export app types (app is private, but pub use works)
pub use app::GetLoaderVersionManifestUseCase;
pub use app::GetMinecraftLaunchCommandParams;
pub use app::GetMinecraftLaunchCommandUseCase;
pub use app::GetVersionManifestUseCase;
pub use app::InstallMinecraftParams;
pub use app::InstallMinecraftUseCase;
pub use app::LoaderVersionResolver;
pub use app::LoaderVersionService;
pub use app::MetadataStorage;
pub use app::MinecraftApplicationError;
pub use app::MinecraftDownloader;
pub use app::MinecraftHealthParams;
pub use app::MinecraftHealthService;
pub use app::ModLoaderProcessor;
pub use app::VersionManifestService;
pub use app::get_class_paths;
pub use app::get_class_paths_jar;
pub use app::get_jvm_arguments;
pub use app::get_lib_path;
