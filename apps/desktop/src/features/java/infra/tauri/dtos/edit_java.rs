use std::path::PathBuf;

use aether_core::features::java::{EditJava, InvalidJavaPath, JavaPath};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditJavaDto {
    pub major_version: u32,
    pub path: PathBuf,
}

impl TryFrom<EditJavaDto> for EditJava {
    type Error = InvalidJavaPath;

    fn try_from(value: EditJavaDto) -> Result<Self, Self::Error> {
        Ok(Self {
            major_version: value.major_version,
            path: JavaPath::try_from(value.path.as_path())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use aether_core::features::java::JavaDomainError;

    use super::*;

    fn edit(path: &str) -> Result<EditJava, JavaDomainError> {
        EditJava::try_from(EditJavaDto {
            major_version: 21,
            path: PathBuf::from(path),
        })
        .map_err(JavaDomainError::from)
    }

    #[test]
    fn should_refuse_the_edit_saying_why_when_the_path_is_relative() {
        let err = edit("jdk-21/bin/javaw.exe").unwrap_err();

        assert!(
            matches!(err, JavaDomainError::RelativePath { ref path } if path == "jdk-21/bin/javaw.exe"),
            "{err}"
        );
        assert!(matches!(edit("").unwrap_err(), JavaDomainError::EmptyPath));
    }

    #[test]
    fn should_keep_the_path_of_the_edit_when_it_is_absolute() {
        let path = std::env::temp_dir().join("javaw");

        let edit_java = edit(path.to_str().unwrap()).unwrap();

        assert_eq!(edit_java.path.as_path(), path);
    }
}
