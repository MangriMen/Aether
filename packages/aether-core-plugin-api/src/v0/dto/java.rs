use serde::{Deserialize, Serialize};

/// Description of a Java installation known to the launcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaDto {
    /// Filesystem location of the Java executable or installation.
    pub path: String,
    /// CPU architecture the installation was built for.
    pub architecture: String,
    /// Major Java release number (for example 8, 17 or 21).
    pub major_version: u32,
    /// Full version string reported by the installation.
    pub version: String,
}
