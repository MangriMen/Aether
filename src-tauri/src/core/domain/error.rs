pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Core error")]
    CoreError(#[from] aether_core::Error),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}
