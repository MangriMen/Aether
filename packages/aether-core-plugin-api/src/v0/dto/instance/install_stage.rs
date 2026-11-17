use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum InstanceInstallStageDto {
    Installed,
    Installing,
    PackInstalling,
    NotInstalled,
    /// A value this version of the API does not know: a newer launcher sent it. Read it, never
    /// write it.
    #[serde(other)]
    Unknown,
}
