/// Pointer width of the target as a string: `"64"` or `"32"`.
pub const ARCH_WIDTH: &str = if cfg!(target_pointer_width = "64") {
    "64"
} else {
    "32"
};
