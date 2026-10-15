use serde::{Deserialize, Serialize};

use crate::v0::LoaderVersionPreferenceDto;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum MinecraftErrorDto {
    VersionNotFound {
        version: String,
    },
    VersionForLoaderNotFound {
        loader_version_preference: LoaderVersionPreferenceDto,
    },
    LoaderNotFound {
        loader_version_preference: LoaderVersionPreferenceDto,
    },
    LoaderVersionRequired,
    DefaultLoaderNotFound,
    ProcessorFailed {
        reason: String,
    },
    PreLaunchFailed {
        code: i32,
    },
    PostLaunchFailed {
        code: i32,
    },
    PathNotFound {
        path: String,
        entity_type: String,
    },
    ParseFailed {
        reason: String,
    },
    StorageFailure {
        reason: String,
    },
}
