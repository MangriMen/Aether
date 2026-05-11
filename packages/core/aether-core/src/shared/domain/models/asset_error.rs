#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Source read error: {0}")]
    ReadSource(String),
}
