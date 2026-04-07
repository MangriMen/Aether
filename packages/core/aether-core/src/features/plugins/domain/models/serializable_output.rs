use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableOutput {
    pub status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl SerializableOutput {
    pub fn from_output(output: &std::process::Output) -> Self {
        SerializableOutput {
            status: output.status.code().unwrap_or(0) as u32,
            stdout: output.stdout.clone(),
            stderr: output.stderr.clone(),
        }
    }
}
