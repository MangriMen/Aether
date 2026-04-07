use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use aether_core_plugin_api::v0::CommandDto;

use crate::{
    features::{plugins::PluginError, settings::LocationInfo},
    shared::{domain::SerializableCommand, IoError},
};

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

    let canonical_base = crate::shared::canonicalize(base_dir)?;
    let canonical_host = crate::shared::canonicalize(&host_path)?;

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
    let resolved_program = plugin_path_to_host(id, &command.program, location_info)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| command.program.clone());

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
