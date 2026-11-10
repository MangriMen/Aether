use std::sync::Arc;

use crate::features::instance::InstanceId;

/// New lines of the game's console log, as the byte range of the file they were appended at.
#[derive(Clone, Debug)]
pub struct GameLogEvent {
    pub instance_id: InstanceId,
    pub source_id: String,
    pub start: u64,
    pub end: u64,
    /// Empty when the lines were too many to send: the range then has to be read.
    pub text: Arc<str>,
}
