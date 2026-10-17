use crate::features::minecraft::vanilla::Os;

/// Classpath entries are separated by `;` on the Windows family and `:` everywhere else.
pub fn get_classpath_separator(java_arch: &str) -> &'static str {
    classpath_separator_for(&Os::native_arch(java_arch))
}

fn classpath_separator_for(os: &Os) -> &'static str {
    let is_windows_family = matches!(os, Os::Windows | Os::WindowsArm64);
    if is_windows_family { ";" } else { ":" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_use_semicolon_when_os_is_windows_family() {
        assert_eq!(classpath_separator_for(&Os::Windows), ";");
        assert_eq!(classpath_separator_for(&Os::WindowsArm64), ";");
    }

    #[test]
    fn should_use_colon_when_os_is_not_windows_family() {
        for os in [
            Os::Osx,
            Os::OsxArm64,
            Os::Linux,
            Os::LinuxArm32,
            Os::LinuxArm64,
            Os::Unknown,
        ] {
            assert_eq!(classpath_separator_for(&os), ":");
        }
    }
}
