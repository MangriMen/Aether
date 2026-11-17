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
}
