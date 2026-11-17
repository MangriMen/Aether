use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContentTypeDto {
    Modpack,
    Mod,
    DataPack,
    ResourcePack,
    ShaderPack,
    /// A value this version of the API does not know: a newer launcher sent it. Read it, never
    /// write it.
    #[serde(other)]
    Unknown,
}
