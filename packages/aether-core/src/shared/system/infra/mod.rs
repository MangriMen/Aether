use sysinfo::System;

pub fn get_total_memory() -> u64 {
    System::new_all().total_memory()
}

/// The OS version in the form Java reports as `os.version`, which version rules of
/// Minecraft manifests are written against: `10.0` on Windows 10 and 11, the release string
/// elsewhere.
#[cfg(windows)]
pub fn get_os_version() -> Option<String> {
    use windows_sys::Wdk::System::SystemServices::RtlGetVersion;
    use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;

    // SAFETY: `info` is zeroed apart from its size field, which is what the call expects.
    let mut info: OSVERSIONINFOW = unsafe { std::mem::zeroed() };
    info.dwOSVersionInfoSize = u32::try_from(std::mem::size_of::<OSVERSIONINFOW>()).ok()?;
    // `RtlGetVersion` is not subject to the compatibility shims that cap `GetVersionEx`.
    let status = unsafe { RtlGetVersion(&raw mut info) };
    (status == 0).then(|| format!("{}.{}", info.dwMajorVersion, info.dwMinorVersion))
}

#[cfg(not(windows))]
pub fn get_os_version() -> Option<String> {
    sys_info::os_release().ok()
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn should_report_windows_10_and_11_as_java_does() {
        let version = get_os_version().expect("RtlGetVersion should succeed");

        assert_eq!(version, "10.0");
    }
}
