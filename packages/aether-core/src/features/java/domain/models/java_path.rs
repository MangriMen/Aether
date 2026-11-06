use std::{
    ffi::OsStr,
    fmt,
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::features::java::JavaDomainError;

/// The path of a Java runtime's executable, as it is stored and launched.
///
/// It must be absolute: a relative path would resolve against whatever the launcher's working
/// directory is when the game starts. A bare command name (`java`) looked up on `PATH` is not
/// accepted either. See [`JavaPathRejection`] for what is refused.
///
/// Equality compares the text as stored: on Windows two spellings of the same file (case,
/// `\\?\` prefix) are different values, so no ordering or hashing is offered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaPath(String);

/// Why a text is not a [`JavaPath`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum JavaPathRejection {
    #[error("it is empty")]
    Empty,
    #[error("it contains a NUL character")]
    Nul,
    #[error("it is not absolute")]
    Relative,
}

/// A text refused by [`JavaPath::parse`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid Java path {value:?}: {reason}")]
pub struct InvalidJavaPath {
    pub value: String,
    pub reason: JavaPathRejection,
}

fn check(value: &str) -> Result<(), JavaPathRejection> {
    if value.is_empty() {
        return Err(JavaPathRejection::Empty);
    }
    if value.contains('\0') {
        return Err(JavaPathRejection::Nul);
    }
    if !Path::new(value).is_absolute() {
        return Err(JavaPathRejection::Relative);
    }
    Ok(())
}

impl JavaPath {
    pub fn parse(value: &str) -> Result<Self, InvalidJavaPath> {
        Self::try_from(value.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl TryFrom<String> for JavaPath {
    type Error = InvalidJavaPath;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match check(&value) {
            Ok(()) => Ok(Self(value)),
            Err(reason) => Err(InvalidJavaPath { value, reason }),
        }
    }
}

/// A path that is not UTF-8 is read lossily, as it always was when stored.
impl TryFrom<&Path> for JavaPath {
    type Error = InvalidJavaPath;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string_lossy().into_owned())
    }
}

impl From<JavaPath> for String {
    fn from(path: JavaPath) -> Self {
        path.0
    }
}

impl From<JavaPath> for PathBuf {
    fn from(path: JavaPath) -> Self {
        PathBuf::from(path.0)
    }
}

impl From<InvalidJavaPath> for JavaDomainError {
    fn from(err: InvalidJavaPath) -> Self {
        match err.reason {
            JavaPathRejection::Empty => JavaDomainError::EmptyPath,
            JavaPathRejection::Relative => JavaDomainError::RelativePath { path: err.value },
            // No file has a NUL in its path, so there is no JRE there.
            JavaPathRejection::Nul => JavaDomainError::InvalidPath {
                path: PathBuf::from(err.value),
            },
        }
    }
}

impl Deref for JavaPath {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for JavaPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for JavaPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<OsStr> for JavaPath {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

impl fmt::Display for JavaPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_a_path_when_it_is_empty_or_relative() {
        for (path, reason) in [
            ("", JavaPathRejection::Empty),
            ("java", JavaPathRejection::Relative),
            ("bin/java", JavaPathRejection::Relative),
            ("./bin/javaw.exe", JavaPathRejection::Relative),
            ("/usr/bin/\0java", JavaPathRejection::Nul),
        ] {
            let err = JavaPath::parse(path).unwrap_err();
            assert_eq!(err.reason, reason, "{path:?}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn should_accept_a_path_when_it_is_a_windows_absolute_or_verbatim_path() {
        for path in [
            r"C:\Program Files\Java\jdk-21\bin\javaw.exe",
            r"\\?\C:\Program Files\Java\jdk-21\bin\javaw.exe",
            r"\\server\share\jdk\bin\javaw.exe",
        ] {
            let java_path = JavaPath::parse(path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
            assert_eq!(java_path.as_str(), path);
        }
        assert_eq!(
            JavaPath::parse(r"\jdk\bin\javaw.exe").unwrap_err().reason,
            JavaPathRejection::Relative
        );
    }

    #[cfg(unix)]
    #[test]
    fn should_accept_a_path_when_it_is_a_unix_absolute_path() {
        let path = "/usr/lib/jvm/java-21/bin/java";

        assert_eq!(JavaPath::parse(path).unwrap().as_str(), path);
    }
}
