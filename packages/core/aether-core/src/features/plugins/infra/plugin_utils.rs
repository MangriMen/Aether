use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use aether_core_plugin_api::v0::CommandDto;

use crate::{
    features::{plugins::PluginError, settings::LocationInfo},
    shared::{io::domain::IoError, serializable_command::domain::SerializableCommand},
};

/// Host environment variables a plugin-started process keeps (T-0.5).
///
/// `env_clear()` on its own breaks the JVM on Windows — it resolves system DLLs through
/// `SystemRoot` — so the child gets this explicit minimal set instead of the launcher's full
/// environment.
#[cfg(windows)]
const INHERITED_ENV_VARS: &[&str] = &[
    "COMSPEC",
    "NUMBER_OF_PROCESSORS",
    "OS",
    "PATH",
    "PATHEXT",
    "PROCESSOR_ARCHITECTURE",
    "SystemDrive",
    "SystemRoot",
    "TEMP",
    "TMP",
    "windir",
];

#[cfg(not(windows))]
const INHERITED_ENV_VARS: &[&str] = &["HOME", "LANG", "LC_ALL", "PATH", "TMPDIR", "TZ"];

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

/// Convert a WASI path (`/mnt/d/path/to/file`) back to a Windows path (`D:\path\to\file`).
/// On non-Windows, returns the path as-is.
pub fn from_wasi_path(input: &str) -> String {
    if cfg!(not(windows)) {
        return input.to_owned();
    }

    let path = input.replace('\\', "/");

    // Match /mnt/<letter>/...
    if let Some(rest) = path.strip_prefix("/mnt/") {
        if let Some(drive_end) = rest.find('/') {
            let drive_letter = &rest[..drive_end];
            let path_after = &rest[drive_end + 1..];
            format!(
                "{}:\\{}",
                drive_letter.to_uppercase(),
                path_after.replace('/', "\\")
            )
        } else {
            // Just /mnt/<letter>
            format!("{}:\\", rest.to_uppercase())
        }
    } else {
        path
    }
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
    // Convert WASI /mnt/<letter>/ paths back to Windows paths
    // so the host can use them (e.g., in run_command)
    let path = if path.starts_with("/mnt/") {
        from_wasi_path(path)
    } else {
        path.to_owned()
    };

    if !path.starts_with('#') {
        return Ok(PathBuf::from(path));
    }

    let cleaned_path_str: String = path.strip_prefix('#').unwrap_or(&path).to_owned();
    let cleaned_path_start_segment = get_first_segment(&cleaned_path_str);

    let allowed_paths = get_default_allowed_paths(location_info, id);
    let plugin_to_host = invert_allowed_paths(&allowed_paths);

    let base_dir = plugin_to_host
        .get(cleaned_path_start_segment)
        .ok_or_else(|| PluginError::AccessViolation {
            plugin_id: id.to_owned(),
            path: path.clone(),
        })?;

    if !base_dir.is_dir() {
        std::fs::create_dir_all(base_dir).map_err(|e| IoError::with_path(e, path))?;
    }

    let stripped_path = plugin_path_to_relative(id, &cleaned_path_str, plugin_to_host.keys())?;
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

/// Build the minimal environment a plugin-started process runs with.
///
/// Values are copied from the launcher's own environment; a variable that is unset here stays
/// unset in the child.
pub fn isolated_env() -> BTreeMap<String, String> {
    INHERITED_ENV_VARS
        .iter()
        .filter_map(|name| {
            std::env::var(name)
                .ok()
                .map(|value| ((*name).to_owned(), value))
        })
        .collect()
}

fn access_violation(id: &str, path: impl AsRef<Path>) -> PluginError {
    PluginError::AccessViolation {
        plugin_id: id.to_owned(),
        path: path.as_ref().to_string_lossy().to_string(),
    }
}

/// Canonicalise or refuse. A path that cannot be resolved — it does not exist, or it is not
/// reachable — is never given the benefit of the doubt.
fn canonicalize_or_deny(id: &str, path: &Path) -> Result<PathBuf, PluginError> {
    crate::shared::io::infra::canonicalize(path).map_err(|_| access_violation(id, path))
}

/// Resolve `program` to the one executable a plugin is allowed to start (T-0.5, Q7).
///
/// The allowlist in this pass is a single directory: the Java tree the core manages, i.e. what
/// `get_java` / `install_java` hand out. Matching on the file name (`java`, `java.exe`) is
/// explicitly **not** a basis — only the path counts, and it is compared after canonicalisation
/// so neither `..` segments nor symlinks can walk out of the tree.
fn resolve_allowed_program(
    id: &str,
    program: &str,
    location_info: &LocationInfo,
) -> Result<String, PluginError> {
    // F-13: no fallback to the raw string. A `program` that does not resolve is refused
    // outright instead of being launched as written.
    let resolved = plugin_path_to_host(id, program, location_info)?;

    let canonical_program = canonicalize_or_deny(id, &resolved)?;
    // Fails closed: if the core has never installed a Java, there is no allowed program.
    let canonical_java_dir = canonicalize_or_deny(id, &location_info.java_dir())?;

    if !canonical_program.starts_with(&canonical_java_dir) {
        return Err(access_violation(id, &canonical_program));
    }

    // Hand back the path that was actually checked, not the one that was asked for.
    Ok(canonical_program.to_string_lossy().to_string())
}

/// Resolve the working directory, which is mandatory and has to sit inside a directory that is
/// really mounted for this plugin (`/cache`, `/instances`).
fn resolve_required_current_dir(
    id: &str,
    current_dir: Option<&Path>,
    location_info: &LocationInfo,
) -> Result<PathBuf, PluginError> {
    let current_dir = current_dir.ok_or_else(|| access_violation(id, "<no current_dir>"))?;

    let resolved = plugin_path_to_host_from_path(id, current_dir, location_info)?;
    let canonical = canonicalize_or_deny(id, &resolved)?;

    let is_mounted = get_default_allowed_paths(location_info, id)
        .keys()
        .filter_map(|root| crate::shared::io::infra::canonicalize(root).ok())
        .any(|root| canonical.starts_with(&root));

    if !is_mounted {
        return Err(access_violation(id, &canonical));
    }

    Ok(canonical)
}

pub fn plugin_command_to_host(
    id: &str,
    command: &CommandDto,
    location_info: &LocationInfo,
) -> Result<SerializableCommand, PluginError> {
    let resolved_program = resolve_allowed_program(id, &command.program, location_info)?;

    let resolved_args: Vec<String> = command
        .args
        .iter()
        .map(|arg| {
            plugin_path_to_host(id, arg, location_info).map(|p| p.to_string_lossy().to_string())
        })
        .collect::<Result<_, PluginError>>()?;

    let resolved_current_dir =
        resolve_required_current_dir(id, command.current_dir.as_deref(), location_info)?;

    Ok(SerializableCommand {
        program: resolved_program,
        args: resolved_args,
        current_dir: Some(resolved_current_dir),
        env: Some(isolated_env()),
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
    use super::*;
    use super::{from_wasi_path, to_wasi_path};

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

    // ── from_wasi_path tests ──

    #[test]
    fn test_from_wasi_path_full() {
        assert_eq!(
            from_wasi_path("/mnt/d/Documents/Minecraft/Test/simple/pack.toml"),
            r"D:\Documents\Minecraft\Test\simple\pack.toml"
        );
    }

    #[test]
    fn test_from_wasi_path_drive_only() {
        assert_eq!(from_wasi_path("/mnt/c"), r"C:\");
        assert_eq!(from_wasi_path("/mnt/D"), r"D:\");
    }

    #[test]
    fn test_from_wasi_path_linux() {
        assert_eq!(from_wasi_path("/home/user/file.txt"), "/home/user/file.txt");
    }

    #[test]
    fn test_from_wasi_path_relative() {
        assert_eq!(
            from_wasi_path("relative/path/file.txt"),
            "relative/path/file.txt"
        );
    }

    #[test]
    fn test_from_wasi_path_empty() {
        assert_eq!(from_wasi_path(""), "");
    }

    #[test]
    fn test_roundtrip() {
        let original = r"D:\Documents\Minecraft\Test\simple\pack.toml";
        let wasi = to_wasi_path(original);
        let back = from_wasi_path(&wasi);
        assert_eq!(back, original);
    }

    // ── plugin_command_to_host: the run_command allowlist (T-0.5, Q7) ──

    const PLUGIN_ID: &str = "packwiz";
    const INSTANCE_ID: &str = "test-instance";

    /// A layout that mirrors a real install: a core-managed Java, a plugin cache holding the
    /// packwiz bootstrap jar, and one instance directory.
    struct Layout {
        _root: tempfile::TempDir,
        location_info: LocationInfo,
        java_bin: PathBuf,
    }

    fn java_file_name() -> &'static str {
        if cfg!(windows) { "java.exe" } else { "java" }
    }

    fn layout() -> Layout {
        let root = tempfile::tempdir().expect("temp dir");
        let config_dir = root.path().join("config");

        let location_info = LocationInfo::new(root.path().join("settings"), config_dir);

        let java_bin_dir = location_info.java_dir().join("zulu-8").join("bin");
        std::fs::create_dir_all(&java_bin_dir).expect("java bin dir");
        let java_bin = java_bin_dir.join(java_file_name());
        std::fs::write(&java_bin, b"").expect("java binary");

        let plugin_cache_dir = location_info.plugin_cache_dir(PLUGIN_ID);
        std::fs::create_dir_all(&plugin_cache_dir).expect("plugin cache dir");
        std::fs::write(
            plugin_cache_dir.join("packwiz-installer-bootstrap.jar"),
            b"",
        )
        .expect("bootstrap jar");

        std::fs::create_dir_all(location_info.instance_dir(INSTANCE_ID)).expect("instance dir");

        Layout {
            _root: root,
            location_info,
            java_bin,
        }
    }

    /// The shape packwiz actually sends: an absolute Java path from `get_java`, `#`-prefixed
    /// paths for everything that lives in a mounted directory.
    fn packwiz_command(program: &str) -> CommandDto {
        CommandDto {
            program: program.to_owned(),
            args: vec![
                "-jar".to_owned(),
                "#/cache/packwiz-installer-bootstrap.jar".to_owned(),
                "--bootstrap-no-update".to_owned(),
                "https://example.com/pack.toml".to_owned(),
            ],
            current_dir: Some(PathBuf::from(format!("#/instances/{INSTANCE_ID}"))),
        }
    }

    #[test]
    fn should_accept_a_host_managed_java_path() {
        let layout = layout();
        let command = packwiz_command(&layout.java_bin.to_string_lossy());

        let host_command =
            plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info).expect("allowed");

        let canonical_java = crate::shared::io::infra::canonicalize(&layout.java_bin).unwrap();
        assert_eq!(
            PathBuf::from(&host_command.program),
            canonical_java,
            "the checked, canonical path must be the one that gets executed"
        );
    }

    #[test]
    fn should_keep_resolving_hash_paths_in_args_and_current_dir() {
        let layout = layout();
        let command = packwiz_command(&layout.java_bin.to_string_lossy());

        let host_command =
            plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info).expect("allowed");

        assert_eq!(host_command.args[0], "-jar");
        assert!(
            PathBuf::from(&host_command.args[1]).ends_with("packwiz-installer-bootstrap.jar"),
            "`#/cache/...` must still resolve to the host cache: {}",
            host_command.args[1]
        );
        assert_eq!(host_command.args[2], "--bootstrap-no-update");
        assert_eq!(host_command.args[3], "https://example.com/pack.toml");

        let current_dir = host_command.current_dir.expect("current_dir is mandatory");
        assert!(
            current_dir.ends_with(INSTANCE_ID),
            "`#/instances/...` must still resolve to the instance: {}",
            current_dir.display()
        );
    }

    #[test]
    fn should_reject_an_arbitrary_program() {
        let layout = layout();

        for program in [
            r"C:\Windows\System32\cmd.exe",
            "/bin/sh",
            "cmd.exe",
            "sh",
            "#/cache/packwiz-installer-bootstrap.jar",
        ] {
            let command = packwiz_command(program);
            assert!(
                matches!(
                    plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info),
                    Err(PluginError::AccessViolation { .. })
                ),
                "`{program}` must be refused"
            );
        }
    }

    #[test]
    fn should_reject_a_java_named_program_outside_the_managed_java_dir() {
        let layout = layout();

        // Same file name, different tree: Q7 says the name is not a basis, the path is.
        let rogue_dir = layout.location_info.config_dir().join("rogue").join("bin");
        std::fs::create_dir_all(&rogue_dir).expect("rogue dir");
        let rogue_java = rogue_dir.join(java_file_name());
        std::fs::write(&rogue_java, b"").expect("rogue java");

        let command = packwiz_command(&rogue_java.to_string_lossy());
        assert!(matches!(
            plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_reject_a_traversal_out_of_the_java_dir() {
        let layout = layout();

        let rogue_dir = layout.location_info.config_dir().join("rogue");
        std::fs::create_dir_all(&rogue_dir).expect("rogue dir");
        let rogue_java = rogue_dir.join(java_file_name());
        std::fs::write(&rogue_java, b"").expect("rogue java");

        let traversal = layout
            .location_info
            .java_dir()
            .join("..")
            .join("..")
            .join("rogue")
            .join(java_file_name());

        let command = packwiz_command(&traversal.to_string_lossy());
        assert!(matches!(
            plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_reject_a_program_that_does_not_exist() {
        let layout = layout();
        let missing = layout.location_info.java_dir().join("zulu-8").join("nope");

        let command = packwiz_command(&missing.to_string_lossy());
        assert!(
            matches!(
                plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info),
                Err(PluginError::AccessViolation { .. })
            ),
            "an unresolvable program must be refused, not launched as written (F-13)"
        );
    }

    #[test]
    fn should_reject_a_missing_current_dir() {
        let layout = layout();
        let mut command = packwiz_command(&layout.java_bin.to_string_lossy());
        command.current_dir = None;

        assert!(matches!(
            plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_reject_a_current_dir_outside_the_mounted_directories() {
        let layout = layout();

        let outside = layout.location_info.config_dir().join("elsewhere");
        std::fs::create_dir_all(&outside).expect("outside dir");

        let mut command = packwiz_command(&layout.java_bin.to_string_lossy());
        command.current_dir = Some(outside);

        assert!(matches!(
            plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_hand_the_child_an_explicit_environment() {
        let layout = layout();
        let command = packwiz_command(&layout.java_bin.to_string_lossy());

        let host_command =
            plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info).expect("allowed");

        let env = host_command
            .env
            .expect("a plugin-started process must not inherit the launcher environment");
        assert!(
            env.keys()
                .all(|name| INHERITED_ENV_VARS.contains(&name.as_str())),
            "only the minimal set may be passed through: {:?}",
            env.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    #[cfg(windows)]
    fn should_keep_system_root_so_the_jvm_still_starts() {
        // env_clear() without SystemRoot breaks JVM startup on Windows.
        assert!(isolated_env().contains_key("SystemRoot"));
    }
}
