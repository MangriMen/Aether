use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableOutputDto {
    pub status: u32,
    #[serde_as(as = "Bytes")]
    pub stdout: Vec<u8>,
    #[serde_as(as = "Bytes")]
    pub stderr: Vec<u8>,
}
