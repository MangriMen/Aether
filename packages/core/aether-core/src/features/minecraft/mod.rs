mod app;
mod domain;
pub(crate) mod infra;

// Re-export domain types explicitly (domain is private, but pub use works from private modules)
pub use domain::LaunchSettings;
pub use domain::LoaderVersionPreference;
pub use domain::MinecraftDomainError;
pub use domain::ModLoader;
pub use domain::TEMPORARY_REPLACE_CHAR;
pub use domain::get_compatible_java_version;
pub use domain::modded;
pub use domain::parse_arguments;
pub use domain::parse_rules;
pub use domain::resolve_minecraft_version;
pub use domain::vanilla;

// Re-export app types (app is private, but pub use works)
pub use app::GetLoaderVersionManifestUseCase;
pub use app::GetMinecraftLaunchCommandParams;
pub use app::GetMinecraftLaunchCommandUseCase;
pub use app::GetVersionManifestUseCase;
pub use app::InstallMinecraftParams;
pub use app::InstallMinecraftUseCase;
pub use app::LoaderVersionResolver;
pub use app::MetadataStorage;
pub use app::MinecraftApplicationError;
pub use app::MinecraftDownloader;
pub use app::ModLoaderProcessor;
pub use app::get_class_paths;
pub use app::get_class_paths_jar;
pub use app::get_jvm_arguments;
pub use app::get_lib_path;
