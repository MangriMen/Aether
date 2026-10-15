use crate::features::minecraft::vanilla;

pub fn get_compatible_java_version(version_info: &vanilla::VersionInfo) -> u32 {
    version_info
        .java_version
        .as_ref()
        .map_or(8, |it| it.major_version)
}
