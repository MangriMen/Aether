use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use aether_core_plugin_api::v0::CommandDto;

use crate::{
    features::{plugins::PluginError, settings::LocationInfo},
    shared::{io::domain::IoError, serializable_command::domain::SerializableCommand},
};

/// Convert a Windows absolute path (e.g. `D:\path\to\file`) to a WASI-compatible
/// path (`/mnt/d/path/to/file`). Non-Windows paths are returned as-is with `/` normalization.
pub fn to_wasi_path(input: &str) -> String {
    let path = input.replace('\\', "/");

    // Check for Windows drive letter pattern (e.g., "C:/" or "d:/")
    let path = if path.len() >= 2
        && path.as_bytes()[1] == b':'
        && (path.len() == 2 || path.as_bytes()[2] == b'/')
    {
        let drive_letter = path.as_bytes()[0].to_ascii_lowercase() as char;
        let rest = if path.len() > 3 { &path[3..] } else { "" };
        format!("/mnt/{drive_letter}/{rest}")
    } else {
        path
    };

    // Collapse double slashes and trailing slash
    let mut result = String::with_capacity(path.len());
    for c in path.chars() {
        if c == '/' && result.ends_with('/') {
            continue;
        }
        result.push(c);
    }
    result.trim_end_matches('/').to_string()
}

pub fn get_default_allowed_paths(
    location_info: &LocationInfo,
    plugin_id: &str,
) -> HashMap<String, PathBuf> {
    HashMap::from([
        (
            location_info
                .plugin_cache_dir(plugin_id)
                .to_string_lossy()
                .to_string(),
            PathBuf::from("/cache".to_owned()),
        ),
        (
            location_info.instances_dir().to_string_lossy().to_string(),
            PathBuf::from("/instances"),
        ),
    ])
}

pub fn invert_allowed_paths(allowed: &HashMap<String, PathBuf>) -> HashMap<String, PathBuf> {
    allowed
        .iter()
        .map(|(host, plugin)| (plugin.to_string_lossy().to_string(), PathBuf::from(host)))
        .collect()
}

pub fn plugin_path_to_relative<I, T>(
    id: &str,
    path: &str,
    allowed_prefixes: I,
) -> Result<PathBuf, PluginError>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let prefix = allowed_prefixes
        .into_iter()
        .find(|prefix| path.starts_with(prefix.as_ref()))
        .ok_or(PluginError::AccessViolation {
            plugin_id: id.to_owned(),
            path: path.to_owned(),
        })?;

    let stripped = path.strip_prefix(prefix.as_ref()).unwrap_or(path);

    Ok(PathBuf::from(
        stripped.strip_prefix('/').unwrap_or(stripped),
    ))
}

pub fn get_first_segment(path: &str) -> &str {
    let mut indices = path.match_indices('/').skip(1);
    if let Some((idx, _)) = indices.next() {
        &path[..idx]
    } else {
        path
    }
}

pub fn plugin_path_to_host(
    id: &str,
    path: &str,
    location_info: &LocationInfo,
) -> Result<PathBuf, PluginError> {
    if !path.starts_with('#') {
        return Ok(PathBuf::from(path));
    }

    let cleaned_path_str = path.strip_prefix('#').unwrap_or(path);
    let cleaned_path_start_segment = get_first_segment(cleaned_path_str);

    let allowed_paths = get_default_allowed_paths(location_info, id);
    let plugin_to_host = invert_allowed_paths(&allowed_paths);

    let base_dir =
        plugin_to_host
            .get(cleaned_path_start_segment)
            .ok_or(PluginError::AccessViolation {
                plugin_id: id.to_owned(),
                path: path.to_owned(),
            })?;

    if !base_dir.is_dir() {
        std::fs::create_dir_all(base_dir).map_err(|e| IoError::with_path(e, path))?;
    }

    let stripped_path = plugin_path_to_relative(id, cleaned_path_str, plugin_to_host.keys())?;
    let host_path = base_dir.join(stripped_path);

    let canonical_base = crate::shared::io::infra::canonicalize(base_dir)?;
    let canonical_host = crate::shared::io::infra::canonicalize(&host_path)?;

    if !canonical_host.starts_with(&canonical_base) {
        return Err(PluginError::AccessViolation {
            plugin_id: id.to_owned(),
            path: canonical_host.to_string_lossy().to_string(),
        });
    }

    Ok(host_path)
}

pub fn plugin_path_to_host_from_path(
    id: &str,
    path: &Path,
    location_info: &LocationInfo,
) -> Result<PathBuf, PluginError> {
    plugin_path_to_host(id, path.to_string_lossy().as_ref(), location_info)
}

pub fn plugin_command_to_host(
    id: &str,
    command: &CommandDto,
    location_info: &LocationInfo,
) -> Result<SerializableCommand, PluginError> {
    let resolved_program = plugin_path_to_host(id, &command.program, location_info).map_or_else(
        |_| command.program.clone(),
        |p| p.to_string_lossy().to_string(),
    );

    let resolved_args: Vec<String> = command
        .args
        .iter()
        .map(|arg| {
            plugin_path_to_host(id, arg, location_info).map(|p| p.to_string_lossy().to_string())
        })
        .collect::<Result<_, PluginError>>()?;

    let resolved_current_dir = command
        .current_dir
        .as_ref()
        .map(|current_dir| plugin_path_to_host_from_path(id, current_dir, location_info))
        .transpose()?;

    Ok(SerializableCommand {
        program: resolved_program,
        args: resolved_args,
        current_dir: resolved_current_dir,
    })
}

pub fn log_level_from_u32(level: u32) -> log::Level {
    match level {
        1 => log::Level::Error,
        2 => log::Level::Warn,
        3 => log::Level::Info,
        4 => log::Level::Debug,
        _ => log::Level::Trace,
    }
}

#[cfg(test)]
mod tests {
    use super::to_wasi_path;

    #[test]
    fn test_windows_drive_with_backslash() {
        assert_eq!(
            to_wasi_path(r"D:\Documents\Minecraft\Test\simple\pack.toml"),
            "/mnt/d/Documents/Minecraft/Test/simple/pack.toml"
        );
    }

    #[test]
    fn test_windows_drive_with_slash() {
        assert_eq!(
            to_wasi_path("C:/Users/test/file.txt"),
            "/mnt/c/Users/test/file.txt"
        );
    }

    #[test]
    fn test_windows_drive_root() {
        assert_eq!(to_wasi_path("D:\\"), "/mnt/d");
    }

    #[test]
    fn test_already_wasi_path() {
        assert_eq!(to_wasi_path("/mnt/c/path/to/file"), "/mnt/c/path/to/file");
    }

    #[test]
    fn test_linux_absolute_path() {
        assert_eq!(to_wasi_path("/home/user/file.txt"), "/home/user/file.txt");
    }

    #[test]
    fn test_relative_path() {
        assert_eq!(
            to_wasi_path("relative/path/file.txt"),
            "relative/path/file.txt"
        );
    }

    #[test]
    fn test_windows_drive_lowercase() {
        assert_eq!(to_wasi_path("d:/path/to/file"), "/mnt/d/path/to/file");
    }

    #[test]
    fn test_trailing_slash() {
        assert_eq!(to_wasi_path("C:\\Users\\"), "/mnt/c/Users");
    }

    #[test]
    fn test_double_slashes() {
        assert_eq!(to_wasi_path("C://path//to//file"), "/mnt/c/path/to/file");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(to_wasi_path(""), "");
    }

    #[test]
    fn test_no_drive_letter_just_colon() {
        assert_eq!(to_wasi_path("some:path"), "some:path");
    }
}
