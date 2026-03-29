#[derive(Debug, thiserror::Error)]
pub enum ModrinthMapperError {
    #[error("Unknown project type received from Modrinth: {0}")]
    UnknownProjectType(String),

    #[error("Missing required field in Modrinth response: {0}")]
    MissingField(String),
}
