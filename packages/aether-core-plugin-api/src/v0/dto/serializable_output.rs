use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

#[serde_as]
#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone)]
pub struct SerializableOutputDto {
    pub status: u32,
    #[serde_as(as = "Bytes")]
    #[schemars(schema_with = "crate::v0::schema::bytes")]
    pub stdout: Vec<u8>,
    #[serde_as(as = "Bytes")]
    #[schemars(schema_with = "crate::v0::schema::bytes")]
    pub stderr: Vec<u8>,
}
