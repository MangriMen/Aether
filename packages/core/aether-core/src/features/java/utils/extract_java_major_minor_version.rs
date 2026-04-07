use crate::features::java::JavaDomainError;

/// Extracts the major and minor version from a Java version string.
///
/// If the string doesn't contain a minor version, it assumes 1 for the major version.
///
/// Examples:
/// - "1.8.0_361" -> (1, 8)
/// - "20" -> (1, 20)
pub fn extract_java_major_minor_version(version: &str) -> Result<(u32, u32), JavaDomainError> {
    let get_error = || JavaDomainError::InvalidVersion {
        version: version.to_string(),
    };

    let mut split = version.split('.');

    let major_str = split.next().ok_or_else(get_error)?;
    let major = major_str.parse::<u32>().map_err(|_| get_error())?;

    // Java start should always be 1. If more than 1, it is formatted like "17.0.1.2" and starts with minor version
    // Formatted like "20", only one value means that is minor version
    if major > 1 {
        Ok((1, major))
    } else {
        let minor_str = split.next().ok_or_else(get_error)?;
        let minor = minor_str.parse::<u32>().map_err(|_| get_error())?;
        Ok((major, minor))
    }
}
