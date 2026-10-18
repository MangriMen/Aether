//! Every size and budget limit of content installation and of the plugin sandbox, in one place.
//!
//! Built in the composition root (`build_container`); the plugin archive extractor has its own,
//! `ZipPluginExtractorConstants`. The timeouts and retries of plugin calls, commands, downloads
//! and HTTP requests are here too.

use std::time::Duration;

/// Limits on what installing content may bring in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentInstallLimits {
    /// Most files the overrides of one plan may list, over all its sources together, so that
    /// splitting an archive of millions of tiny entries across sources does not multiply it.
    pub max_override_files: usize,
    /// Most bytes the overrides of one plan may stage, counted as written: the sizes an entry
    /// declares are the archive's word, and a zip bomb understates them.
    pub max_override_bytes: u64,
    /// The largest `.mrpack` downloaded, as for any other content file.
    pub max_mrpack_bytes: u64,
    /// Largest content file an install plan may bring, the same as a plugin's `download_to_cache`.
    pub max_content_bytes: u64,
    /// Most files one install plan may list, before any is resolved or downloaded. Its
    /// overrides are bounded on their own (`max_override_files`).
    pub max_plan_files: usize,
    /// Most bytes the files of one install plan may download together, counted as they arrive:
    /// a file's declared size is the provider's word. Its overrides are bounded on their own
    /// (`max_override_bytes`).
    pub max_plan_bytes: u64,
}

impl Default for ContentInstallLimits {
    fn default() -> Self {
        Self {
            max_override_files: 100_000,
            max_override_bytes: 4 * 1024 * 1024 * 1024,
            max_mrpack_bytes: 1024 * 1024 * 1024,
            max_content_bytes: 1024 * 1024 * 1024,
            max_plan_files: MAX_MRPACK_FILES,
            max_plan_bytes: 8 * 1024 * 1024 * 1024,
        }
    }
}

/// Largest `modrinth.index.json` read out of a `.mrpack`, counted as decompressed: the entry's
/// declared size is the archive's word, and a small archive can inflate to gigabytes of JSON.
/// Real indexes of packs with hundreds of mods stay well under a megabyte.
pub const MAX_MRPACK_INDEX_BYTES: u64 = 8 * 1024 * 1024;

/// Most files a `.mrpack` index may list: each becomes a download of the plan.
pub const MAX_MRPACK_FILES: usize = 10_000;

/// Limits on what a plugin may do through the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginSandboxLimits {
    /// Hard cap on the response body a plugin may receive in one `http_get` /
    /// `http_post_json` call.
    pub max_response_bytes: usize,
    /// Hard cap on the JSON body a plugin may send in one `http_post_json` call.
    pub max_request_body_bytes: usize,
    /// Most redirects a GET follows. Being redirected once more is an error.
    pub max_redirects: usize,
    /// Largest file a plugin may download in one call (and the core for a plugin), so that a
    /// hostile or broken server cannot fill the disk.
    pub max_download_bytes: u64,
    /// Hard cap on how much of each stream (stdout, stderr) a plugin gets back from
    /// `run_command`. Everything past it is read and dropped, so the child never blocks on a
    /// full pipe; a stream that hit it is reported as truncated.
    pub max_output_bytes: usize,
    /// Largest file a plugin may write into, or read out of, an instance in one call. The bytes
    /// pass through the plugin's memory [`INSTANCE_FILE_MEMORY_COPIES`] times over, so it must
    /// fit that many times in [`Self::default_memory_limit_bytes`] ([`Self::assert_consistent`]);
    /// for a plugin with a memory limit of its own see [`Self::max_instance_file_bytes_within`].
    pub max_instance_file_bytes: u64,
    /// Memory limit for a plugin whose manifest does not declare one (256 MiB). A manifest's own
    /// value takes precedence.
    pub default_memory_limit_bytes: usize,
    /// Instruction budget for a single plugin call.
    pub default_fuel_limit: u64,
}

impl Default for PluginSandboxLimits {
    fn default() -> Self {
        Self {
            max_response_bytes: 16 * 1024 * 1024,
            max_request_body_bytes: 4 * 1024 * 1024,
            max_redirects: 5,
            max_download_bytes: 1024 * 1024 * 1024,
            max_output_bytes: 1024 * 1024,
            max_instance_file_bytes: 64 * 1024 * 1024,
            default_memory_limit_bytes: 256 * 1024 * 1024,
            default_fuel_limit: 1_000_000_000,
        }
    }
}

/// How many copies of a file's bytes a plugin's linear memory holds at once while it passes
/// through `read_instance_file` / `write_instance_file`: the plugin's buffer, the encoded
/// message and the decoded one.
pub const INSTANCE_FILE_MEMORY_COPIES: u64 = 3;

/// Fallback execution timeout for a single plugin call (30 minutes).
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_mins(30);

/// Wall-clock budget for one `run_command`. Generous, since the packwiz installer downloads a
/// whole modpack within it: it stops a wedged child, not useful work.
pub(crate) const COMMAND_TIMEOUT: Duration = Duration::from_mins(15);

/// Total time budget for one attempt, every redirect hop and the body included. A stalled
/// transfer is cut much earlier by the client's read timeout.
pub(crate) const DOWNLOAD_TIMEOUT: Duration = Duration::from_mins(30);

/// Time budget for one whole `download_to_cache`: every attempt, the waits between them and the
/// wait for a download slot. Never more than a plugin call may take: Extism cannot stop a call
/// while the host holds it, so a longer download would keep the call running past its timeout.
pub(crate) const DOWNLOAD_BUDGET: Duration = DEFAULT_TIMEOUT;

const _: () = assert!(
    DOWNLOAD_BUDGET.as_nanos() <= DEFAULT_TIMEOUT.as_nanos(),
    "a download must not outlast the plugin call it runs in"
);

/// Retries of transient failures (connection errors, 5xx, 429). Fewer than the application
/// client's five: the plugin waits on this call for the whole time.
pub(crate) const DOWNLOAD_MAX_RETRIES: u32 = 3;

/// Total time budget for one `http_get`, including connect and body download.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Time budget for establishing the connection alone.
pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Longest a connection may stay silent mid-response. Only matters for calls whose total
/// budget is longer than this, i.e. `download_to_cache`.
pub(crate) const READ_TIMEOUT: Duration = Duration::from_secs(30);

impl PluginSandboxLimits {
    /// Refuses limits that contradict each other, rather than letting a plugin find out at run
    /// time: a file the host accepts must fit, [`INSTANCE_FILE_MEMORY_COPIES`] times over, in the
    /// memory a plugin gets by default.
    ///
    /// # Panics
    ///
    /// When `max_instance_file_bytes` times [`INSTANCE_FILE_MEMORY_COPIES`] is not below
    /// `default_memory_limit_bytes`.
    pub fn assert_consistent(&self) {
        assert!(
            self.max_instance_file_bytes
                .saturating_mul(INSTANCE_FILE_MEMORY_COPIES)
                < self.default_memory_limit_bytes as u64,
            "max_instance_file_bytes ({}) times {INSTANCE_FILE_MEMORY_COPIES} must stay below \
             default_memory_limit_bytes ({}): instance file bytes pass through the plugin's \
             linear memory",
            self.max_instance_file_bytes,
            self.default_memory_limit_bytes
        );
    }

    /// The largest file a plugin may move through an instance given the memory limit its
    /// manifest declares (`None`: the default one): [`Self::max_instance_file_bytes`], or less
    /// when the plugin's memory could not hold the copies of it. A file over it is refused as
    /// too large instead of running the plugin out of memory.
    pub fn max_instance_file_bytes_within(&self, declared_memory_limit: Option<usize>) -> u64 {
        let memory_limit = declared_memory_limit.unwrap_or(self.default_memory_limit_bytes);

        self.max_instance_file_bytes
            .min(memory_limit as u64 / INSTANCE_FILE_MEMORY_COPIES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_the_default_limits_consistent_when_checked_together() {
        PluginSandboxLimits::default().assert_consistent();
    }

    #[test]
    #[should_panic(expected = "must stay below")]
    fn should_refuse_an_instance_file_cap_when_it_does_not_fit_the_plugin_memory() {
        PluginSandboxLimits {
            max_instance_file_bytes: 256 * 1024 * 1024,
            ..PluginSandboxLimits::default()
        }
        .assert_consistent();
    }

    #[test]
    #[should_panic(expected = "must stay below")]
    fn should_refuse_an_instance_file_cap_when_it_fits_the_plugin_memory_only_once() {
        PluginSandboxLimits {
            max_instance_file_bytes: 128 * 1024 * 1024,
            ..PluginSandboxLimits::default()
        }
        .assert_consistent();
    }

    #[test]
    fn should_keep_the_instance_file_cap_when_the_plugin_has_the_default_memory() {
        let limits = PluginSandboxLimits::default();

        assert_eq!(
            limits.max_instance_file_bytes_within(None),
            limits.max_instance_file_bytes
        );
    }

    #[test]
    fn should_lower_the_instance_file_cap_when_the_manifest_declares_less_memory() {
        let limits = PluginSandboxLimits::default();

        assert_eq!(
            limits.max_instance_file_bytes_within(Some(30 * 1024 * 1024)),
            10 * 1024 * 1024
        );
    }

    #[test]
    fn should_keep_the_instance_file_cap_when_the_manifest_declares_more_memory() {
        let limits = PluginSandboxLimits::default();

        assert_eq!(
            limits.max_instance_file_bytes_within(Some(1024 * 1024 * 1024)),
            limits.max_instance_file_bytes
        );
    }
}
