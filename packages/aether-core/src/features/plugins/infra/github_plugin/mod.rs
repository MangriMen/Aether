pub(super) mod fetcher;
pub(super) mod provider;
pub(super) mod types;

pub use fetcher::GitHubPluginFetcher;
pub use provider::GithubProvider;
pub use types::{GitHubPluginPreview, GitHubReleaseInfo, PluginUpdateInfo};
