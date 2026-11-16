use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::ContentFileDto;

/// A log level, as the plugin names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevelDto {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// The input of `log`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogParamsDto {
    pub level: LogLevelDto,
    pub message: String,
}

/// The input of `get_java` and `install_java`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersionParamsDto {
    /// The major Java release (for example 8, 17 or 21).
    pub version: u32,
}

/// The input of the host functions that take one instance: `instance_get_dir`,
/// `instance_plugin_get_dir` and `list_content`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceIdParamsDto {
    pub instance_id: String,
}

/// The input of `enable_contents` and `disable_contents`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeContentsParamsDto {
    pub instance_id: String,
    /// The paths of the files, as `list_content` names them.
    pub paths: Vec<String>,
}

/// The answer of `instance_get_dir` and `instance_plugin_get_dir`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceDirDto {
    /// A path in the plugin's own file system (`/instances/…`).
    pub path: String,
}

/// The answer of `instance_create`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceCreatedDto {
    pub instance_id: String,
}

/// The answer of `list_content`.
#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContentResultDto {
    /// By the path of the file in the instance.
    pub files: HashMap<String, ContentFileDto>,
}
