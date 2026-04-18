use crate::features::minecraft::vanilla::Os;

pub fn get_classpath_separator(java_arch: &str) -> &'static str {
    match Os::native_arch(java_arch) {
        Os::Osx | Os::OsxArm64 | Os::Linux | Os::LinuxArm32 | Os::LinuxArm64 | Os::Unknown => ":",
        Os::Windows | Os::WindowsArm64 => ";",
    }
}
