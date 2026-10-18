/// WebAssembly linear memory page size (64 KiB).
pub(super) const WASM_PAGE_SIZE_BYTES: usize = 64 * 1024;

/// Convert a byte limit into WebAssembly pages, rounding up and saturating at `u32::MAX`.
pub(super) fn bytes_to_pages(bytes: usize) -> u32 {
    u32::try_from(bytes.div_ceil(WASM_PAGE_SIZE_BYTES)).unwrap_or(u32::MAX)
}
