use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableOutputDto {
    pub status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
