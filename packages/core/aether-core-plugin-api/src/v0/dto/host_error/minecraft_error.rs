use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum MinecraftErrorDto {
    VersionNotFound { version: String },
    VersionForLoaderNotFound { loader_version_preference: String },
    LoaderNotFound { loader_version_preference: String },
    LoaderVersionRequired,
    DefaultLoaderNotFound,
    ProcessorFailed { reason: String },
    PreLaunchFailed { code: i32 },
    PostLaunchFailed { code: i32 },
    PathNotFound { path: String, entity_type: String },
    ParseFailed { reason: String },
    StorageFailure { reason: String },
}
