//! `InstallPlanExecutor` against mock providers and a recording downloader: refs resolved
//! through the provider chosen for their platform, each URL downloaded under the policy of the
//! provider that supplied it, and nothing in the instance until every file is downloaded.
//!
//! The network policies themselves — a plugin's URL refused outside its `allowed_hosts`, a hash
//! mismatch — are exercised with the real downloader in `plugins::tests::install_plan_download_test`.

mod change_game;
mod common;
mod diff;
mod install;
mod one_version;
mod overrides;
mod resolve;
mod rollback;
