use crate::features::minecraft::vanilla::Os;

/// Classpath entries are separated by `;` on the Windows family and `:` everywhere else.
pub fn get_classpath_separator(java_arch: &str) -> &'static str {
    classpath_separator_for(&Os::native_arch(java_arch))
}

fn classpath_separator_for(os: &Os) -> &'static str {
    match os {
        Os::Osx | Os::OsxArm64 | Os::Linux | Os::LinuxArm32 | Os::LinuxArm64 | Os::Unknown => ":",
        Os::Windows | Os::WindowsArm64 => ";",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::windows(Os::Windows, ";")]
    #[case::windows_arm64(Os::WindowsArm64, ";")]
    #[case::osx(Os::Osx, ":")]
    #[case::osx_arm64(Os::OsxArm64, ":")]
    #[case::linux(Os::Linux, ":")]
    #[case::linux_arm32(Os::LinuxArm32, ":")]
    #[case::linux_arm64(Os::LinuxArm64, ":")]
    #[case::unknown(Os::Unknown, ":")]
    fn should_pick_the_classpath_separator_when_given_the_os(
        #[case] os: Os,
        #[case] expected: &str,
    ) {
        assert_eq!(classpath_separator_for(&os), expected);
    }
}
