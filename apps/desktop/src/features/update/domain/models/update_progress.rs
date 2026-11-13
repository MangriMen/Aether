use super::UpdateError;

#[derive(Debug, Clone)]
pub struct UpdateProgress {
    pub fraction: Option<f64>,
    pub phase: UpdatePhase,
    pub version: String,
    pub current_version: String,
}

#[derive(Debug, Clone)]
pub enum UpdatePhase {
    Started,
    Progress,
    Finished,
    /// The install failed; the error is the one the command that started it returns.
    Error(UpdateError),
}
