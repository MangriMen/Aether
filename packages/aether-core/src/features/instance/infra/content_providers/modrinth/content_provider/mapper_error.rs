#[derive(Debug, thiserror::Error)]
pub enum ModrinthMapperError {
    #[error("Unknown project type received from Modrinth: {0}")]
    UnknownProjectType(String),

    #[error("Missing required field in Modrinth response: {0}")]
    MissingField(String),

    #[error("Invalid id received from Modrinth: {0}")]
    InvalidId(#[from] crate::features::instance::InvalidContentId),

    #[error("Invalid URL received from Modrinth: {0}")]
    InvalidUrl(#[from] crate::shared::web_url::InvalidUrl),
}
