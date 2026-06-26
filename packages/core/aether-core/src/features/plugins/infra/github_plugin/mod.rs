pub mod fetcher;
pub mod provider;
pub mod types;

pub use fetcher::GitHubPluginFetcher;
pub use provider::GithubProvider;
pub use types::{GitHubPluginPreview, GitHubReleaseInfo, PluginUpdateInfo};
