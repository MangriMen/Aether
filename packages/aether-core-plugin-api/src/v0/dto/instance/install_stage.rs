use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceInstallStageDto {
    Installed,
    Installing,
    PackInstalling,
    NotInstalled,
}
