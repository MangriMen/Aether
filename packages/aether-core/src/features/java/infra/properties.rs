use std::{path::Path, process::Command};

use crate::features::java::JavaDomainError;

#[derive(Debug, Default)]
pub struct JavaProperties {
    pub version: Option<String>,
    pub architecture: Option<String>,
}

pub fn get_java_properties(path: &Path) -> Result<JavaProperties, JavaDomainError> {
    let output = Command::new(path)
        .arg("-XshowSettings:properties")
        .arg("-version")
        .env_remove("_JAVA_OPTIONS")
        .output()
        .map_err(|source| JavaDomainError::FailedToGetProperties {
            path: path.to_path_buf(),
            source,
        })?;

    let mut combined_output = String::new();
    combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
    combined_output.push_str(&String::from_utf8_lossy(&output.stderr));

    let mut version = None;
    let mut architecture = None;

    for line in combined_output.lines() {
        let (key, value) = line.split_once('=').unwrap_or(("", ""));

        match key.trim() {
            "os.arch" => architecture = Some(value.trim().to_string()),
            "java.version" => version = Some(value.trim().to_string()),
            _ => {}
        }
    }

    Ok(JavaProperties {
        version,
        architecture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_the_path_and_the_io_source_when_java_cannot_be_started() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("no-java");

        let err = get_java_properties(&missing).unwrap_err();

        assert!(matches!(
            err,
            JavaDomainError::FailedToGetProperties { ref path, ref source }
                if path == &missing && source.kind() == std::io::ErrorKind::NotFound
        ));
    }
}
