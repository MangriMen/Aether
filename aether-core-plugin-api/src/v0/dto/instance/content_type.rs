use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentTypeDto {
    Modpack,
    Mod,
    DataPack,
    ResourcePack,
    ShaderPack,
}
