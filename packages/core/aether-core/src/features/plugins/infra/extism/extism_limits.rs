use std::time::Duration;

/// WebAssembly linear memory page size (64 KiB).
pub const WASM_PAGE_SIZE_BYTES: usize = 64 * 1024;

/// Fallback memory limit used when the plugin manifest does not declare one (256 MiB).
pub const DEFAULT_MEMORY_LIMIT_BYTES: usize = 256 * 1024 * 1024;

/// Fallback execution timeout for a single plugin call (30 minutes).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_mins(30);

/// Fallback instruction budget for a single plugin call.
pub const DEFAULT_FUEL_LIMIT: u64 = 1_000_000_000;

/// Convert a byte limit into WebAssembly pages, rounding up and saturating at `u32::MAX`.
pub fn bytes_to_pages(bytes: usize) -> u32 {
    u32::try_from(bytes.div_ceil(WASM_PAGE_SIZE_BYTES)).unwrap_or(u32::MAX)
}
