pub fn get_compatible_java_version(version_info: &daedalus::minecraft::VersionInfo) -> u32 {
    version_info
        .java_version
        .as_ref()
        .map(|it| it.major_version)
        .unwrap_or(8)
}
