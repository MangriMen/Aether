use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};
use tokio::fs::ReadDir;

use crate::shared::io::domain::IoError;

// Bytes

pub async fn read_async(path: impl AsRef<Path>) -> Result<Vec<u8>, IoError> {
    let path_ref = path.as_ref();
    tokio::fs::read(path_ref)
        .await
        .map_err(|err| IoError::with_path(err, path_ref))
}

pub async fn write_async(path: impl AsRef<Path>, data: impl AsRef<[u8]>) -> Result<(), IoError> {
    let path_ref = path.as_ref();

    if let Some(parent) = path_ref.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| IoError::with_path(err, parent))?;
    }

    tokio::fs::write(path_ref, data)
        .await
        .map_err(|err| IoError::with_path(err, path_ref))
}

/// How the temporary files of [`write_replacing_async`] start, for a cleanup to recognise the
/// leftovers of an interrupted write.
pub(crate) const REPLACING_TEMP_PREFIX: &str = ".write-";

/// Replaces the file as a whole or not at all: a crash or a full disk mid-write leaves the
/// previous content, never a truncated file that no longer parses.
///
/// The temporary file sits in the same directory, so the final step is a rename within one
/// file system, which the OS performs atomically.
///
/// `data` is taken as it is (a `Vec`, or shared `Bytes`), not copied.
pub(crate) async fn write_replacing_async(
    path: &Path,
    data: impl AsRef<[u8]> + Send + 'static,
) -> Result<(), IoError> {
    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };

    tokio::fs::create_dir_all(&parent)
        .await
        .map_err(|err| IoError::with_path(err, &parent))?;

    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        // Dropping `temp_file` on any early return removes it.
        let mut temp_file = tempfile::Builder::new()
            .prefix(REPLACING_TEMP_PREFIX)
            .suffix(".tmp")
            .tempfile_in(&parent)
            .map_err(|err| IoError::with_path(err, &parent))?;

        std::io::Write::write_all(&mut temp_file, data.as_ref())
            .and_then(|()| temp_file.as_file().sync_all())
            .map_err(|err| IoError::with_path(err, temp_file.path()))?;

        temp_file
            .persist(&path)
            .map_err(|err| IoError::with_path(err.error, &path))?;

        Ok(())
    })
    .await
    .map_err(|err| IoError::from(std::io::Error::other(err)))?
}

// JSON

pub async fn read_json_async<T>(path: impl AsRef<Path>) -> Result<T, IoError>
where
    T: DeserializeOwned,
{
    let bytes = read_async(&path).await?;
    serde_json::from_slice(&bytes).map_err(|e| IoError::DeserializationError(e.to_string()))
}

pub async fn write_json_async<T>(path: impl AsRef<Path>, data: T) -> Result<(), IoError>
where
    T: Serialize,
{
    let bytes =
        serde_json::to_vec(&data).map_err(|e| IoError::SerializationError(e.to_string()))?;
    write_replacing_async(path.as_ref(), bytes).await
}

pub async fn ensure_read_json_async<T>(path: impl AsRef<Path>) -> Result<T, IoError>
where
    T: Serialize + DeserializeOwned + Default,
{
    let path_ref = path.as_ref();

    if !path_ref.exists() {
        let default = T::default();
        write_json_async(path, &default).await?;
        return Ok(default);
    }

    read_json_async(path).await
}

// TOML

pub async fn read_toml_async<T>(path: impl AsRef<Path>) -> Result<T, IoError>
where
    T: DeserializeOwned,
{
    let bytes = read_async(&path).await?;
    let toml_str =
        std::str::from_utf8(&bytes).map_err(|e| IoError::DeserializationError(e.to_string()))?;

    toml::from_str(toml_str).map_err(|e| IoError::DeserializationError(e.to_string()))
}

pub async fn write_toml_async<T>(path: impl AsRef<Path>, data: T) -> Result<(), IoError>
where
    T: Serialize,
{
    let toml_str =
        toml::to_string(&data).map_err(|e| IoError::SerializationError(e.to_string()))?;
    write_replacing_async(path.as_ref(), toml_str.into_bytes()).await
}

pub async fn ensure_read_toml_async<T>(path: impl AsRef<Path>) -> Result<T, IoError>
where
    T: Serialize + DeserializeOwned + Default,
{
    let path_ref = path.as_ref();

    if !path_ref.exists() {
        let default = T::default();
        write_toml_async(path, &default).await?;
        return Ok(default);
    }

    read_toml_async(path).await
}

// dunce canonicalize

pub fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf, IoError> {
    let path_ref = path.as_ref();
    dunce::canonicalize(path_ref).map_err(|e| IoError::with_path(e, path_ref))
}

pub async fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), IoError> {
    let from_ref = from.as_ref();
    tokio::fs::rename(from_ref, to.as_ref())
        .await
        .map_err(|e| IoError::with_path(e, from_ref))
}

pub async fn remove_file(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path_ref = path.as_ref();
    tokio::fs::remove_file(path_ref)
        .await
        .map_err(|e| IoError::with_path(e, path_ref))
}

pub async fn create_dir_all(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path_ref = path.as_ref();
    tokio::fs::create_dir_all(path_ref)
        .await
        .map_err(|e| IoError::with_path(e, path_ref))
}

pub async fn remove_dir_all(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path_ref = path.as_ref();
    tokio::fs::remove_dir_all(path_ref)
        .await
        .map_err(|e| IoError::with_path(e, path_ref))
}

pub async fn read_dir(path: impl AsRef<Path>) -> Result<ReadDir, IoError> {
    let path_ref = path.as_ref();
    tokio::fs::read_dir(path_ref)
        .await
        .map_err(|e| IoError::with_path(e, path_ref))
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), IoError> {
    std::fs::create_dir_all(&dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dst_path = dst.as_ref().join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(entry.path(), dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Record {
        value: String,
    }

    fn record(value: &str) -> Record {
        Record {
            value: value.to_owned(),
        }
    }

    fn file_names(dir: &Path) -> Vec<String> {
        std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect()
    }

    #[tokio::test]
    async fn should_replace_json_and_leave_no_temp_file_when_written_over_an_existing_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("record.json");

        write_json_async(&path, record("old")).await.unwrap();
        write_json_async(&path, record("new")).await.unwrap();

        assert_eq!(
            read_json_async::<Record>(&path).await.unwrap(),
            record("new")
        );
        assert_eq!(file_names(dir.path()), vec!["record.json".to_owned()]);
    }

    #[tokio::test]
    async fn should_replace_toml_and_leave_no_temp_file_when_written_over_an_existing_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("record.toml");

        write_toml_async(&path, record("old")).await.unwrap();
        write_toml_async(&path, record("new")).await.unwrap();

        assert_eq!(
            read_toml_async::<Record>(&path).await.unwrap(),
            record("new")
        );
        assert_eq!(
            file_names(path.parent().unwrap()),
            vec!["record.toml".to_owned()]
        );
    }

    #[tokio::test]
    async fn should_leave_no_temp_file_when_the_target_cannot_be_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("record.json");
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("inside"), b"").unwrap();

        let result = write_json_async(&path, record("new")).await;

        assert!(result.is_err());
        assert_eq!(file_names(dir.path()), vec!["record.json".to_owned()]);
    }
}
