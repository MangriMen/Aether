//! Every limit on size or count, and every timeout, that protects against external input
//! (network, archives, plugin calls, game output), in one place, each with what it protects
//! against. Adapters import them; none declares its own.
//!
//! The grouped limits are built in the composition root (`build_container`); the plugin archive
//! extractor has its own, `ZipPluginExtractorConstants`. Not here: a bound that is part of a
//! value type's invariant, and the settings of a feature that do not guard against outside
//! input (concurrency, batching, paging, cache lifetimes).

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
    /// `http_post_json` call; for a plugin with a memory limit of its own see
    /// [`Self::max_response_bytes_within`].
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
    /// pass through the plugin's memory [`PLUGIN_MEMORY_COPIES`] times over, so it must
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

/// How many copies of the bytes a plugin's linear memory holds at once while they pass through
/// `read_instance_file` / `write_instance_file`, or come back from `http_get` /
/// `http_post_json`: the plugin's buffer, the encoded message and the decoded one.
pub const PLUGIN_MEMORY_COPIES: u64 = 3;

/// Fallback execution timeout for a single plugin call (30 minutes).
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_mins(30);

/// Wall-clock budget for one `run_command`. Generous, since the packwiz installer downloads a
/// whole modpack within it: it stops a wedged child, not useful work.
pub(crate) const COMMAND_TIMEOUT: Duration = Duration::from_mins(15);

/// Time budget for one probe of a Java runtime (`java -version`), which only prints its
/// properties: a runtime that has not answered by then is wedged, and must not hold a worker.
pub(crate) const JAVA_PROBE_TIMEOUT: Duration = Duration::from_secs(15);

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
pub(crate) const PLUGIN_HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Time budget for establishing the connection alone.
pub(crate) const PLUGIN_HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Longest a connection may stay silent mid-response. Only matters for calls whose total
/// budget is longer than this, i.e. `download_to_cache`.
pub(crate) const PLUGIN_HTTP_READ_TIMEOUT: Duration = Duration::from_secs(30);

impl PluginSandboxLimits {
    /// Refuses limits that contradict each other, rather than letting a plugin find out at run
    /// time: a file the host accepts must fit, [`PLUGIN_MEMORY_COPIES`] times over, in the
    /// memory a plugin gets by default.
    ///
    /// # Panics
    ///
    /// When `max_instance_file_bytes` times [`PLUGIN_MEMORY_COPIES`] is not below
    /// `default_memory_limit_bytes`.
    pub fn assert_consistent(&self) {
        assert!(
            self.max_instance_file_bytes
                .saturating_mul(PLUGIN_MEMORY_COPIES)
                < self.default_memory_limit_bytes as u64,
            "max_instance_file_bytes ({}) times {PLUGIN_MEMORY_COPIES} must stay below \
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
            .min(memory_limit as u64 / PLUGIN_MEMORY_COPIES)
    }

    /// The largest response body `http_get` / `http_post_json` hand a plugin given the memory
    /// limit its manifest declares (`None`: the default one): [`Self::max_response_bytes`], or
    /// less when the plugin's memory could not hold the copies of it. A body over it fails the
    /// call as too large instead of running the plugin out of memory.
    pub fn max_response_bytes_within(&self, declared_memory_limit: Option<usize>) -> usize {
        let memory_limit = declared_memory_limit.unwrap_or(self.default_memory_limit_bytes);
        let per_copy = memory_limit as u64 / PLUGIN_MEMORY_COPIES;

        usize::try_from(per_copy).map_or(self.max_response_bytes, |per_copy| {
            self.max_response_bytes.min(per_copy)
        })
    }
}

/// How long the application's HTTP client may take to open a connection.
pub(crate) const HTTP_CLIENT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// How long a response to the application's HTTP client may stay silent. Per read, not per
/// request: a large download that keeps flowing is never cut, a stalled one is. Generous, so a
/// poor connection is not mistaken for a dead one; the retry middleware covers the rest.
pub(crate) const HTTP_CLIENT_READ_TIMEOUT: Duration = Duration::from_secs(90);

/// Retries of transient failures (connection errors, 5xx, 429) by the application's HTTP
/// client. The plugins' downloads retry less: see [`DOWNLOAD_MAX_RETRIES`].
pub const HTTP_CLIENT_MAX_RETRIES: u32 = 5;

/// Upper bound for a downloaded plugin archive. The body goes to disk as it arrives, so this
/// stops an endless stream rather than limiting what a plugin may weigh.
pub(crate) const MAX_PLUGIN_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;

/// The largest content icon taken from a provider. Modrinth's are a few KiB.
pub(crate) const MAX_ICON_BYTES: u64 = 1 << 20;

/// How long fetching one content icon may take.
pub(crate) const ICON_FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// How many redirects a content icon fetch follows.
pub(crate) const MAX_ICON_REDIRECTS: usize = 3;

/// The largest manifest read from a jar; a real one is a few lines.
pub(crate) const MAX_JAR_MANIFEST_BYTES: u64 = 1024 * 1024;

/// The largest jar that may be approved for good; hashing a bigger one for every run is not worth
/// it, and a tool that size is not what a plugin ships.
pub const MAX_APPROVED_JAR_BYTES: u64 = 256 * 1024 * 1024;

/// A plugin's README past this is not shown: the text goes to the UI whole, and a plugin's
/// archive is not the launcher's to trust.
pub(crate) const MAX_PLUGIN_README_BYTES: u64 = 256 * 1024;

/// The longest author ref the host keeps and sends back to a plugin; it is an opaque token (a
/// team or organization id), so a longer one is a plugin's mistake, not an author.
pub(crate) const MAX_PLUGIN_AUTHOR_REF_LEN: usize = 256;

/// The most characters of an author name the host keeps from a plugin's content provider.
pub(crate) const MAX_PLUGIN_AUTHOR_NAME_CHARS: usize = 128;

/// The largest file `import_file` takes. The source may be a file a plugin wrote anywhere it can
/// write, so it is not read whole before its size is known.
pub(crate) const MAX_ASSET_BYTES: u64 = 4 * 1024 * 1024;

/// How long a pre-launch hook may run. The instance stays locked meanwhile, and a hook that
/// never ends would keep the game from starting for good.
pub(crate) const PRELAUNCH_HOOK_TIMEOUT: Duration = Duration::from_mins(10);

/// Most a game's console log file may grow to, so a game that prints without end cannot fill the
/// disk; what comes after is dropped with a note in the file.
pub(crate) const MAX_GAME_LOG_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// A packed game log that unpacks to more than this is refused, so a crafted file cannot fill
/// the disk.
pub(crate) const MAX_GAME_LOG_UNPACKED_BYTES: u64 = 256 * 1024 * 1024;

/// The longest line of game output read as one; a longer one is split, so a game that never
/// prints a newline cannot grow a buffer without bound.
pub(crate) const MAX_GAME_LOG_LINE_BYTES: usize = 16 * 1024;

/// A game log event larger than this is written out as the raw lines it arrived in.
pub(crate) const MAX_GAME_LOG_EVENT_BYTES: usize = 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn should_keep_the_default_limits_consistent_when_checked_together() {
        PluginSandboxLimits::default().assert_consistent();
    }

    #[rstest]
    #[case::does_not_fit_the_plugin_memory(256 * 1024 * 1024)]
    #[case::fits_the_plugin_memory_only_once(128 * 1024 * 1024)]
    #[should_panic(expected = "must stay below")]
    fn should_refuse_an_instance_file_cap_when_it_does_not_leave_room(#[case] cap: u64) {
        PluginSandboxLimits {
            max_instance_file_bytes: cap,
            ..PluginSandboxLimits::default()
        }
        .assert_consistent();
    }

    #[rstest]
    #[case::default_memory(None, None)]
    #[case::manifest_declares_less_memory(Some(30 * 1024 * 1024), Some(10 * 1024 * 1024))]
    #[case::manifest_declares_more_memory(Some(1024 * 1024 * 1024), None)]
    fn should_size_the_instance_file_cap_when_the_manifest_declares_memory(
        #[case] declared_memory: Option<usize>,
        #[case] expected: Option<u64>,
    ) {
        let limits = PluginSandboxLimits::default();

        assert_eq!(
            limits.max_instance_file_bytes_within(declared_memory),
            expected.unwrap_or(limits.max_instance_file_bytes)
        );
    }

    #[rstest]
    #[case::default_memory(None, None)]
    #[case::manifest_declares_less_memory(Some(30 * 1024 * 1024), Some(10 * 1024 * 1024))]
    #[case::manifest_declares_just_enough_memory(Some(48 * 1024 * 1024), None)]
    #[case::manifest_declares_more_memory(Some(1024 * 1024 * 1024), None)]
    fn should_size_the_response_cap_when_the_manifest_declares_memory(
        #[case] declared_memory: Option<usize>,
        #[case] expected: Option<usize>,
    ) {
        let limits = PluginSandboxLimits::default();

        assert_eq!(
            limits.max_response_bytes_within(declared_memory),
            expected.unwrap_or(limits.max_response_bytes)
        );
    }
}
