use crate::features::plugins::domain::PluginError;

/// Write bytes to a temporary file and return the file handle.
/// The file is automatically deleted when the returned `NamedTempFile` is dropped.
pub fn write_bytes_to_temp_file(bytes: &[u8]) -> Result<tempfile::NamedTempFile, PluginError> {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new()
        .map_err(|e| PluginError::Storage(crate::shared::io::domain::IoError::IoError(e)))?;

    tmp.write_all(bytes)
        .map_err(|e| PluginError::Storage(crate::shared::io::domain::IoError::IoError(e)))?;

    // Flush and rewind so the reader sees all data
    tmp.flush()
        .map_err(|e| PluginError::Storage(crate::shared::io::domain::IoError::IoError(e)))?;

    Ok(tmp)
}
