use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use aether_core_plugin_api::v0::CommandDto;

pub(crate) use crate::features::plugins::{
    PLUGIN_CACHE_MOUNT, effective_allowed_hosts, to_wasi_path,
};

use crate::{
    features::{
        plugins::{PLUGIN_INSTANCES_MOUNT, PluginError},
        settings::LocationInfo,
    },
    shared::{io::domain::IoError, serializable_command::domain::SerializableCommand},
};

/// The hashing task of a plugin binary did not finish. The cause is logged here, the caller
/// gets the error that tells the user what failed.
pub(crate) fn hash_failed(err: &std::io::Error) -> PluginError {
    tracing::error!("Failed to hash a plugin binary: {err}");
    PluginError::HashComputationFailed
}

/// Host environment variables a plugin-started process keeps instead of the launcher's full
/// environment. Not none at all: the JVM on Windows resolves system DLLs through `SystemRoot`.
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

/// Convert a WASI path (`/mnt/d/path/to/file`) back to a Windows path (`D:\path\to\file`).
/// On non-Windows, returns the path as-is.
pub(crate) fn from_wasi_path(input: &str) -> String {
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

/// Extism's marker for a host directory that is mounted read-only.
///
/// It sits on the host side of the mapping and belongs only in the string handed to the Extism
/// manifest, never in the host↔plugin resolution of this module.
pub(crate) const READ_ONLY_MOUNT_PREFIX: &str = "ro:";

/// The plugin-side mount points the WASM sandbox may only read.
///
/// Every write into an instance goes through `write_instance_file`, the single checked path,
/// which also guards `.metadata`. `/cache` stays read-write: it is the plugin's own scratch space.
const READ_ONLY_PLUGIN_MOUNTS: &[&str] = &[PLUGIN_INSTANCES_MOUNT];

/// Whether `plugin_path` is one of the core's mount points the sandbox may only read. Matches
/// the mount root itself, not a prefix.
pub(crate) fn is_read_only_plugin_mount(plugin_path: &Path) -> bool {
    READ_ONLY_PLUGIN_MOUNTS
        .iter()
        .any(|mount| plugin_path == Path::new(mount))
}

/// The host-side key to hand Extism for one mount, read-only ones carrying [`READ_ONLY_MOUNT_PREFIX`].
///
/// Separate from [`get_default_allowed_paths`]: the prefix in that map would break
/// `invert_allowed_paths` and the directory creation over its keys.
pub(crate) fn to_extism_mount_key(host: &str, read_only: bool) -> String {
    if read_only {
        format!("{READ_ONLY_MOUNT_PREFIX}{host}")
    } else {
        host.to_owned()
    }
}

pub(crate) fn get_default_allowed_paths(
    location_info: &LocationInfo,
    plugin_id: &str,
) -> HashMap<String, PathBuf> {
    HashMap::from([
        (
            location_info
                .plugin_cache_dir(plugin_id)
                .to_string_lossy()
                .to_string(),
            PathBuf::from(PLUGIN_CACHE_MOUNT),
        ),
        (
            location_info.instances_dir().to_string_lossy().to_string(),
            PathBuf::from(PLUGIN_INSTANCES_MOUNT),
        ),
    ])
}

pub(crate) fn invert_allowed_paths(allowed: &HashMap<String, PathBuf>) -> HashMap<String, PathBuf> {
    allowed
        .iter()
        .map(|(host, plugin)| {
            // Defensive: a prefixed key would send `#`-path resolution to a directory literally
            // named `ro:...`.
            let host = host.strip_prefix(READ_ONLY_MOUNT_PREFIX).unwrap_or(host);

            (plugin.to_string_lossy().to_string(), PathBuf::from(host))
        })
        .collect()
}

pub(crate) fn plugin_path_to_relative<I, T>(
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

pub(crate) fn get_first_segment(path: &str) -> &str {
    let mut indices = path.match_indices('/').skip(1);
    if let Some((idx, _)) = indices.next() {
        &path[..idx]
    } else {
        path
    }
}

pub(crate) fn plugin_path_to_host(
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

pub(crate) fn plugin_path_to_host_from_path(
    id: &str,
    path: &Path,
    location_info: &LocationInfo,
) -> Result<PathBuf, PluginError> {
    plugin_path_to_host(id, path.to_string_lossy().as_ref(), location_info)
}

/// The host file behind `plugin_path`, a file in the plugin's own `/cache` for the core to read
/// (such as the overrides of an install plan).
///
/// Stricter than [`plugin_path_to_host`], which takes a path without `#` as a host path and
/// would let a plugin have the core copy any file of the user's. The file must exist and stay
/// inside `/cache` after resolving `..` and symlinks; the canonical path is returned so a
/// symlink swapped in later does not redirect the core.
pub(crate) fn plugin_cache_path_to_host(
    location_info: &LocationInfo,
    plugin_id: &str,
    plugin_path: &str,
) -> Result<PathBuf, PluginError> {
    let relative = plugin_path
        .strip_prefix(PLUGIN_CACHE_MOUNT)
        .filter(|rest| rest.is_empty() || rest.starts_with('/'))
        .ok_or_else(|| access_violation(plugin_id, plugin_path))?;

    let base = location_info.plugin_cache_dir(plugin_id);
    let host = base.join(relative.trim_start_matches('/'));

    let canonical_base = canonicalize_or_deny(plugin_id, &base)?;
    let canonical_host = canonicalize_or_deny(plugin_id, &host)?;

    if !canonical_host.starts_with(&canonical_base) {
        return Err(access_violation(plugin_id, plugin_path));
    }

    Ok(canonical_host)
}

/// The plugin-side path of an existing host file under one of the plugin's mounts (`/cache`,
/// `/instances`); `None` for a file the plugin could not open.
pub(crate) fn host_path_to_plugin(
    location_info: &LocationInfo,
    plugin_id: &str,
    host_path: &Path,
) -> Option<String> {
    let host_path = crate::shared::io::infra::canonicalize(host_path).ok()?;

    get_default_allowed_paths(location_info, plugin_id)
        .into_iter()
        .find_map(|(host_dir, mount)| {
            let base = crate::shared::io::infra::canonicalize(Path::new(&host_dir)).ok()?;
            let rest = host_path.strip_prefix(&base).ok()?;

            let mut plugin_path = mount.to_string_lossy().into_owned();
            for component in rest.components() {
                plugin_path.push('/');
                plugin_path.push_str(&component.as_os_str().to_string_lossy());
            }
            Some(plugin_path)
        })
}

/// The minimal environment a plugin-started process runs with, copied from the launcher's.
pub(crate) fn isolated_env() -> BTreeMap<String, String> {
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

/// The directories a plugin may start a program from: the canonical `bin` directories of the
/// Java runtimes the core knows about. The directory rather than the file, because `get_java`
/// hands out `javaw` and a caller asking for `java` next to it wants the same runtime.
///
/// A path that no longer resolves is dropped: a stale row must not widen the allowlist.
pub(crate) fn java_bin_dirs<I, S>(registered_java_paths: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<Path>,
{
    let mut dirs: Vec<PathBuf> = registered_java_paths
        .into_iter()
        .filter_map(|path| crate::shared::io::infra::canonicalize(path.as_ref()).ok())
        .map(|path| {
            if path.is_dir() {
                path
            } else {
                path.parent().map_or(path.clone(), Path::to_path_buf)
            }
        })
        .collect();

    dirs.sort();
    dirs.dedup();
    dirs
}

/// Resolve `program` to an executable a plugin is allowed to start: one in a directory of
/// [`java_bin_dirs`], judged by its canonical directory, never by its file name.
///
/// A narrowing (no shell, no arbitrary native binary), not a containment boundary: the plugin
/// chooses the arguments, so `<allowed java> -jar #/cache/…` runs whatever it put there.
fn resolve_allowed_program(
    id: &str,
    program: &str,
    location_info: &LocationInfo,
    allowed_program_dirs: &[PathBuf],
) -> Result<String, PluginError> {
    // No fallback to the raw string: a `program` that does not resolve is refused instead of
    // being launched as written.
    let resolved = plugin_path_to_host(id, program, location_info)?;

    let canonical_program = canonicalize_or_deny(id, &resolved)?;

    // Fails closed: with no Java registered there is no allowed program at all.
    let is_allowed = canonical_program
        .parent()
        .is_some_and(|parent| allowed_program_dirs.iter().any(|dir| dir == parent));

    if !is_allowed {
        return Err(access_violation(id, &canonical_program));
    }

    // The path that was checked, not the one that was asked for.
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

pub(crate) fn plugin_command_to_host(
    id: &str,
    command: &CommandDto,
    location_info: &LocationInfo,
    allowed_program_dirs: &[PathBuf],
) -> Result<SerializableCommand, PluginError> {
    let resolved_program =
        resolve_allowed_program(id, &command.program, location_info, allowed_program_dirs)?;

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

pub(crate) fn log_level_from_u32(level: u32) -> log::Level {
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
    fn should_convert_to_wasi_path_when_windows_drive_has_a_backslash() {
        assert_eq!(
            to_wasi_path(r"D:\Documents\Minecraft\Test\simple\pack.toml"),
            "/mnt/d/Documents/Minecraft/Test/simple/pack.toml"
        );
    }

    #[test]
    fn should_convert_to_wasi_path_when_windows_drive_has_a_slash() {
        assert_eq!(
            to_wasi_path("C:/Users/test/file.txt"),
            "/mnt/c/Users/test/file.txt"
        );
    }

    #[test]
    fn should_convert_to_wasi_path_when_it_is_a_windows_drive_root() {
        assert_eq!(to_wasi_path("D:\\"), "/mnt/d");
    }

    #[test]
    fn should_leave_the_path_as_is_when_it_is_already_wasi() {
        assert_eq!(to_wasi_path("/mnt/c/path/to/file"), "/mnt/c/path/to/file");
    }

    #[test]
    fn should_leave_the_path_as_is_when_it_is_a_linux_absolute_path() {
        assert_eq!(to_wasi_path("/home/user/file.txt"), "/home/user/file.txt");
    }

    #[test]
    fn should_leave_the_path_as_is_when_it_is_relative() {
        assert_eq!(
            to_wasi_path("relative/path/file.txt"),
            "relative/path/file.txt"
        );
    }

    #[test]
    fn should_convert_to_wasi_path_when_the_windows_drive_letter_is_lowercase() {
        assert_eq!(to_wasi_path("d:/path/to/file"), "/mnt/d/path/to/file");
    }

    #[test]
    fn should_drop_the_trailing_slash_when_converting_to_wasi_path() {
        assert_eq!(to_wasi_path("C:\\Users\\"), "/mnt/c/Users");
    }

    #[test]
    fn should_collapse_double_slashes_when_converting_to_wasi_path() {
        assert_eq!(to_wasi_path("C://path//to//file"), "/mnt/c/path/to/file");
    }

    #[test]
    fn should_return_empty_when_the_path_is_empty() {
        assert_eq!(to_wasi_path(""), "");
    }

    #[test]
    fn should_leave_the_path_as_is_when_it_has_a_colon_but_no_drive_letter() {
        assert_eq!(to_wasi_path("some:path"), "some:path");
    }

    // ── from_wasi_path tests ──

    #[test]
    fn should_convert_from_wasi_path_when_it_names_a_drive_and_a_file() {
        assert_eq!(
            from_wasi_path("/mnt/d/Documents/Minecraft/Test/simple/pack.toml"),
            r"D:\Documents\Minecraft\Test\simple\pack.toml"
        );
    }

    #[test]
    fn should_convert_from_wasi_path_when_it_names_only_a_drive() {
        assert_eq!(from_wasi_path("/mnt/c"), r"C:\");
        assert_eq!(from_wasi_path("/mnt/D"), r"D:\");
    }

    #[test]
    fn should_leave_the_path_as_is_when_converting_a_linux_path_from_wasi() {
        assert_eq!(from_wasi_path("/home/user/file.txt"), "/home/user/file.txt");
    }

    #[test]
    fn should_leave_the_path_as_is_when_converting_a_relative_path_from_wasi() {
        assert_eq!(
            from_wasi_path("relative/path/file.txt"),
            "relative/path/file.txt"
        );
    }

    #[test]
    fn should_return_empty_when_converting_an_empty_path_from_wasi() {
        assert_eq!(from_wasi_path(""), "");
    }

    #[test]
    fn should_roundtrip_when_converted_to_wasi_and_back() {
        let original = r"D:\Documents\Minecraft\Test\simple\pack.toml";
        let wasi = to_wasi_path(original);
        let back = from_wasi_path(&wasi);
        assert_eq!(back, original);
    }

    // ── `ro:` on /instances ──

    #[test]
    fn should_mark_only_the_instances_mount_read_only_when_building_mounts() {
        let location_info = LocationInfo::new(PathBuf::from("settings"), PathBuf::from("config"));
        let allowed = get_default_allowed_paths(&location_info, PLUGIN_ID);

        let cache_host = location_info
            .plugin_cache_dir(PLUGIN_ID)
            .to_string_lossy()
            .to_string();
        let instances_host = location_info.instances_dir().to_string_lossy().to_string();

        assert_eq!(
            to_extism_mount_key(
                &instances_host,
                is_read_only_plugin_mount(&allowed[&instances_host])
            ),
            format!("ro:{instances_host}"),
            "the instance tree must reach the sandbox read-only"
        );
        assert_eq!(
            to_extism_mount_key(
                &cache_host,
                is_read_only_plugin_mount(&allowed[&cache_host])
            ),
            cache_host,
            "the plugin cache stays read-write: it is the plugin's own scratch space"
        );
    }

    #[test]
    fn should_keep_the_read_only_prefix_out_of_the_default_mapping_when_it_is_built() {
        // The map itself feeds `create_dir_all` and `#`-path resolution, so a `ro:`-prefixed
        // key there would mean a directory literally named `ro:...`.
        let location_info = LocationInfo::new(PathBuf::from("settings"), PathBuf::from("config"));

        assert!(
            get_default_allowed_paths(&location_info, PLUGIN_ID)
                .keys()
                .all(|host| !host.starts_with(READ_ONLY_MOUNT_PREFIX))
        );
    }

    #[test]
    fn should_invert_a_prefixed_mapping_back_to_plain_host_paths_when_inverting() {
        let location_info = LocationInfo::new(PathBuf::from("settings"), PathBuf::from("config"));
        let allowed = get_default_allowed_paths(&location_info, PLUGIN_ID);

        let prefixed: HashMap<String, PathBuf> = allowed
            .iter()
            .map(|(host, plugin)| {
                (
                    to_extism_mount_key(host, is_read_only_plugin_mount(plugin)),
                    plugin.clone(),
                )
            })
            .collect();

        assert_eq!(
            invert_allowed_paths(&prefixed),
            invert_allowed_paths(&allowed),
            "`ro:` is an Extism manifest detail and must not survive inversion"
        );

        let inverted = invert_allowed_paths(&prefixed);
        assert!(
            inverted.keys().all(|plugin| !plugin.contains(':')),
            "the keys are plugin-side mount points: {:?}",
            inverted.keys().collect::<Vec<_>>()
        );
        assert!(
            inverted
                .values()
                .all(|host| !host.to_string_lossy().starts_with(READ_ONLY_MOUNT_PREFIX)),
            "the values are host paths the core itself opens: {:?}",
            inverted.values().collect::<Vec<_>>()
        );
    }

    // ── plugin_command_to_host: the run_command allowlist ──

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

    impl Layout {
        /// What `handle_run_command` would compute from `java_versions` for this layout.
        fn allowed(&self) -> Vec<PathBuf> {
            java_bin_dirs([self.java_bin.as_path()])
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
    fn should_still_resolve_hash_paths_when_the_mount_is_read_only() {
        // `ro:` narrows WASI only. `write_instance_file` and `run_command` resolve through
        // `plugin_path_to_host`, which has to keep working for `/instances`.
        let layout = layout();

        let resolved = plugin_path_to_host(
            PLUGIN_ID,
            &format!("#/instances/{INSTANCE_ID}"),
            &layout.location_info,
        )
        .expect("an instance path must still resolve");

        assert_eq!(
            crate::shared::io::infra::canonicalize(&resolved).unwrap(),
            crate::shared::io::infra::canonicalize(layout.location_info.instance_dir(INSTANCE_ID))
                .unwrap()
        );
    }

    #[test]
    fn should_accept_a_program_when_it_is_a_host_managed_java_path() {
        let layout = layout();
        let command = packwiz_command(&layout.java_bin.to_string_lossy());

        let host_command = plugin_command_to_host(
            PLUGIN_ID,
            &command,
            &layout.location_info,
            &layout.allowed(),
        )
        .expect("allowed");

        let canonical_java = crate::shared::io::infra::canonicalize(&layout.java_bin).unwrap();
        assert_eq!(
            PathBuf::from(&host_command.program),
            canonical_java,
            "the checked, canonical path must be the one that gets executed"
        );
    }

    #[test]
    fn should_keep_resolving_hash_paths_when_they_are_in_args_and_current_dir() {
        let layout = layout();
        let command = packwiz_command(&layout.java_bin.to_string_lossy());

        let host_command = plugin_command_to_host(
            PLUGIN_ID,
            &command,
            &layout.location_info,
            &layout.allowed(),
        )
        .expect("allowed");

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
    fn should_reject_a_program_when_it_is_arbitrary() {
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
                    plugin_command_to_host(
                        PLUGIN_ID,
                        &command,
                        &layout.location_info,
                        &layout.allowed()
                    ),
                    Err(PluginError::AccessViolation { .. })
                ),
                "`{program}` must be refused"
            );
        }
    }

    #[test]
    fn should_reject_a_program_when_it_is_java_named_but_outside_the_managed_java_dir() {
        let layout = layout();

        // Same file name, different tree: the name is not a basis, the path is.
        let rogue_dir = layout.location_info.config_dir().join("rogue").join("bin");
        std::fs::create_dir_all(&rogue_dir).expect("rogue dir");
        let rogue_java = rogue_dir.join(java_file_name());
        std::fs::write(&rogue_java, b"").expect("rogue java");

        let command = packwiz_command(&rogue_java.to_string_lossy());
        assert!(matches!(
            plugin_command_to_host(
                PLUGIN_ID,
                &command,
                &layout.location_info,
                &layout.allowed()
            ),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_reject_a_program_when_it_traverses_out_of_the_java_dir() {
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
            plugin_command_to_host(
                PLUGIN_ID,
                &command,
                &layout.location_info,
                &layout.allowed()
            ),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_reject_a_program_when_it_does_not_exist() {
        let layout = layout();
        let missing = layout.location_info.java_dir().join("zulu-8").join("nope");

        let command = packwiz_command(&missing.to_string_lossy());
        assert!(
            matches!(
                plugin_command_to_host(
                    PLUGIN_ID,
                    &command,
                    &layout.location_info,
                    &layout.allowed()
                ),
                Err(PluginError::AccessViolation { .. })
            ),
            "an unresolvable program must be refused, not launched as written"
        );
    }

    #[test]
    fn should_reject_a_current_dir_when_it_is_missing() {
        let layout = layout();
        let mut command = packwiz_command(&layout.java_bin.to_string_lossy());
        command.current_dir = None;

        assert!(matches!(
            plugin_command_to_host(
                PLUGIN_ID,
                &command,
                &layout.location_info,
                &layout.allowed()
            ),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_reject_a_current_dir_when_it_is_outside_the_mounted_directories() {
        let layout = layout();

        let outside = layout.location_info.config_dir().join("elsewhere");
        std::fs::create_dir_all(&outside).expect("outside dir");

        let mut command = packwiz_command(&layout.java_bin.to_string_lossy());
        command.current_dir = Some(outside);

        assert!(matches!(
            plugin_command_to_host(
                PLUGIN_ID,
                &command,
                &layout.location_info,
                &layout.allowed()
            ),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_accept_a_java_when_it_is_registered_outside_the_managed_java_dir() {
        // What counts is that the core handed the path out, not where it sits.
        let layout = layout();

        let system_bin = layout
            .location_info
            .config_dir()
            .join("Program Files")
            .join("Eclipse Adoptium")
            .join("jdk-8")
            .join("bin");
        std::fs::create_dir_all(&system_bin).expect("system java dir");
        let system_java = system_bin.join(java_file_name());
        std::fs::write(&system_java, b"").expect("system java");

        let command = packwiz_command(&system_java.to_string_lossy());
        let allowed = java_bin_dirs([system_java.as_path()]);

        assert!(
            plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info, &allowed).is_ok(),
            "a system-wide Java the core registered must be allowed"
        );
    }

    #[test]
    fn should_accept_a_sibling_launcher_when_it_is_in_the_same_bin_directory() {
        // `get_java` hands out `javaw`; a caller asking for `java` next to it wants the same
        // runtime, so the allowlist is the `bin` directory, not the exact file.
        let layout = layout();

        let sibling =
            layout
                .java_bin
                .with_file_name(if cfg!(windows) { "javaw.exe" } else { "javaw" });
        std::fs::write(&sibling, b"").expect("sibling launcher");

        let command = packwiz_command(&sibling.to_string_lossy());
        assert!(
            plugin_command_to_host(
                PLUGIN_ID,
                &command,
                &layout.location_info,
                &layout.allowed()
            )
            .is_ok()
        );
    }

    #[test]
    fn should_reject_a_program_when_it_is_elsewhere_under_the_managed_java_dir() {
        // Being somewhere under `java_dir()` is not enough: the program must sit in a
        // registered runtime's `bin`.
        let layout = layout();

        let stray = layout.location_info.java_dir().join("zulu-8").join("stray");
        std::fs::write(&stray, b"").expect("stray binary");

        let command = packwiz_command(&stray.to_string_lossy());
        assert!(matches!(
            plugin_command_to_host(
                PLUGIN_ID,
                &command,
                &layout.location_info,
                &layout.allowed()
            ),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_reject_everything_when_no_java_is_registered() {
        let layout = layout();
        let command = packwiz_command(&layout.java_bin.to_string_lossy());

        assert!(
            matches!(
                plugin_command_to_host(PLUGIN_ID, &command, &layout.location_info, &[]),
                Err(PluginError::AccessViolation { .. })
            ),
            "an empty allowlist must permit nothing, not everything"
        );
    }

    #[test]
    fn should_drop_registered_paths_when_they_no_longer_resolve() {
        let layout = layout();
        let missing = layout.location_info.java_dir().join("gone").join("bin");

        assert_eq!(
            java_bin_dirs([missing.as_path()]),
            Vec::<PathBuf>::new(),
            "a stale row must not widen the allowlist"
        );
    }

    #[test]
    fn should_normalise_to_the_same_entry_when_a_bin_directory_and_a_binary_are_registered() {
        let layout = layout();
        let bin_dir = layout.java_bin.parent().expect("bin dir");

        assert_eq!(
            java_bin_dirs([layout.java_bin.as_path()]),
            java_bin_dirs([bin_dir]),
            "`java_versions` stores either form; both mean the same runtime"
        );
    }

    #[test]
    fn should_hand_the_child_an_explicit_environment_when_a_command_is_started() {
        let layout = layout();
        let command = packwiz_command(&layout.java_bin.to_string_lossy());

        let host_command = plugin_command_to_host(
            PLUGIN_ID,
            &command,
            &layout.location_info,
            &layout.allowed(),
        )
        .expect("allowed");

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

    // ── Paths in install plans and pack sources ──

    #[test]
    fn should_translate_a_file_when_it_is_in_the_plugin_cache() {
        let layout = layout();
        let expected = crate::shared::io::infra::canonicalize(
            layout
                .location_info
                .plugin_cache_dir(PLUGIN_ID)
                .join("packwiz-installer-bootstrap.jar"),
        )
        .unwrap();

        let host = plugin_cache_path_to_host(
            &layout.location_info,
            PLUGIN_ID,
            "/cache/packwiz-installer-bootstrap.jar",
        )
        .expect("a file in the plugin's own cache");

        assert_eq!(host, expected);
    }

    #[test]
    fn should_refuse_a_path_when_it_is_outside_the_plugin_cache() {
        let layout = layout();
        let instance_dir = layout.location_info.instance_dir(INSTANCE_ID);
        let other_mount = format!("/instances/{INSTANCE_ID}");

        for path in [
            // A bare host path, which `plugin_path_to_host` would take as is.
            layout.java_bin.to_str().unwrap(),
            instance_dir.to_str().unwrap(),
            // The other mount, a lookalike prefix, the `#` form and an escape through `..`.
            other_mount.as_str(),
            "/cachex/packwiz-installer-bootstrap.jar",
            "#/cache/packwiz-installer-bootstrap.jar",
            "/cache/../../../config",
            // Not there: nothing to resolve, so nothing is trusted.
            "/cache/missing.zip",
        ] {
            assert!(
                matches!(
                    plugin_cache_path_to_host(&layout.location_info, PLUGIN_ID, path),
                    Err(PluginError::AccessViolation { .. })
                ),
                "{path} must be refused"
            );
        }
    }

    #[test]
    fn should_name_a_host_file_through_the_plugin_mount_when_the_plugin_can_read_it() {
        let layout = layout();
        let pack = layout
            .location_info
            .instance_dir(INSTANCE_ID)
            .join("pack.toml");
        std::fs::write(&pack, b"").unwrap();

        assert_eq!(
            host_path_to_plugin(&layout.location_info, PLUGIN_ID, &pack).as_deref(),
            Some(format!("/instances/{INSTANCE_ID}/pack.toml").as_str())
        );
        assert_eq!(
            host_path_to_plugin(
                &layout.location_info,
                PLUGIN_ID,
                &layout
                    .location_info
                    .plugin_cache_dir(PLUGIN_ID)
                    .join("packwiz-installer-bootstrap.jar"),
            )
            .as_deref(),
            Some("/cache/packwiz-installer-bootstrap.jar")
        );
    }

    #[test]
    fn should_not_name_a_host_file_when_the_plugin_cannot_read_it() {
        let layout = layout();

        assert_eq!(
            host_path_to_plugin(&layout.location_info, PLUGIN_ID, &layout.java_bin),
            None
        );
        assert_eq!(
            host_path_to_plugin(
                &layout.location_info,
                "another-plugin",
                &layout
                    .location_info
                    .plugin_cache_dir(PLUGIN_ID)
                    .join("packwiz-installer-bootstrap.jar"),
            ),
            None,
            "another plugin's cache is not mounted for this one"
        );
    }

    #[test]
    #[cfg(windows)]
    fn should_keep_system_root_when_building_the_environment_so_the_jvm_still_starts() {
        // env_clear() without SystemRoot breaks JVM startup on Windows.
        assert!(isolated_env().contains_key("SystemRoot"));
    }
}
