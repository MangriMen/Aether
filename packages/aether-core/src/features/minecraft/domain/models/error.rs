use std::path::PathBuf;

use super::LoaderVersionPreference;
use crate::shared::io::domain::IoError;

/// What a path the launch needs is the path of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePathKind {
    Game,
    Assets,
    Library,
    NativeLibrary,
    LogConfig,
    /// The client jar, put on the classpath.
    Classpath,
}

impl GamePathKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::Assets => "assets",
            Self::Library => "library",
            Self::NativeLibrary => "native_library",
            Self::LogConfig => "log_config",
            Self::Classpath => "classpath",
        }
    }
}

impl std::fmt::Display for GamePathKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a modloader processor did not finish.
#[derive(Debug, thiserror::Error)]
pub enum ProcessorFailure {
    #[error("Could not find processor main class for {jar}")]
    MainClassNotFound { jar: String },

    #[error("Argument contains unresolved placeholders: {argument}")]
    UnresolvedPlaceholder { argument: String },

    #[error("Cannot read processor at {path}")]
    CannotReadProcessor { path: String },

    #[error("Cannot read processor manifest at {path}")]
    CannotReadManifest { path: String },

    #[error("Error running processor: {0}")]
    SpawnFailed(#[source] IoError),

    /// `code` is empty when the OS ended the process by a signal. `stderr` is the text of the
    /// processor, not ours.
    #[error("Processor error: {stderr}")]
    Exited { code: Option<i32>, stderr: String },

    #[error("Processor task failed: {0}")]
    TaskFailed(#[source] tokio::task::JoinError),
}

#[derive(Debug, thiserror::Error)]
pub enum MinecraftDomainError {
    #[error("Minecraft version \"{version}\" not found")]
    VersionNotFound { version: String },

    #[error("Minecraft version not found for loader version {loader_version_preference:?}")]
    VersionForLoaderNotFound {
        loader_version_preference: LoaderVersionPreference,
    },

    #[error("Loader version {loader_version_preference:?} not found")]
    LoaderNotFound {
        loader_version_preference: LoaderVersionPreference,
    },

    #[error("Loader version not specified for non-vanilla loader")]
    LoaderVersionRequired,

    #[error("Stable or latest loader could not be resolved")]
    DefaultLoaderNotFound,

    #[error("Modloader processor failed: {reason}")]
    ProcessorFailed { reason: ProcessorFailure },

    #[error("Failed to execute pre-launch command: exit code {code}")]
    PreLaunchFailed { code: i32 },

    #[error("Failed to execute post-launch command: exit code {code}")]
    PostLaunchFailed { code: i32 },

    #[error("Path not found: {path:?} ({entity_type})")]
    PathNotFound {
        path: PathBuf,
        entity_type: GamePathKind,
    },

    /// `reason` is the text of the parser (daedalus), not ours.
    #[error("Error while parsing libraries: {reason}")]
    ParseFailed { reason: String },

    #[error("Storage failure: {0}")]
    StorageFailure(#[source] IoError),

    /// The log config of a version is named so that it would leave its directory.
    #[error("Invalid log config name: {name}")]
    InvalidLogConfigName { name: String },
}
