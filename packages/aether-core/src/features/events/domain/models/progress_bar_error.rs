use uuid::Uuid;

#[derive(Clone, Debug, thiserror::Error)]
pub enum ProgressBarStorageError {
    #[error("Progress bar {progress_bar_id} already exists")]
    AlreadyExists { progress_bar_id: Uuid },

    #[error("Progress bar {progress_bar_id} not found")]
    NotFound { progress_bar_id: Uuid },
}
