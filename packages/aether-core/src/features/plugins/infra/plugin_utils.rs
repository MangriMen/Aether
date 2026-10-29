use std::{
    collections::{BTreeMap, HashMap},
    path::{Component, Path, PathBuf, Prefix},
};

use aether_core_plugin_api::v0::CommandDto;

pub(crate) use crate::features::plugins::{
    PLUGIN_CACHE_MOUNT, effective_allowed_hosts, to_wasi_path,
};

use crate::{
    features::{
        instance::PluginId,
        plugins::{CoreMount, PluginError},
        settings::LocationInfo,
    },
    shared::serializable_command::domain::SerializableCommand,
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

/// The host-side key to hand Extism for one mount, read-only ones carrying [`READ_ONLY_MOUNT_PREFIX`].
///
/// Separate from [`get_default_allowed_paths`]: the prefix in that map would end up in the
/// directory creation over its keys.
pub(crate) fn to_extism_mount_key(host: &str, read_only: bool) -> String {
    if read_only {
        format!("{READ_ONLY_MOUNT_PREFIX}{host}")
    } else {
        host.to_owned()
    }
}

/// The mounts a plugin path may resolve into. Each caller names the narrowest set it needs: a
/// file the core will copy into the asset store has no business in `/instances`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MountSet(&'static [CoreMount]);

impl MountSet {
    pub(crate) const CACHE: Self = Self(&[CoreMount::Cache]);
    pub(crate) const ALL: Self = Self(&CoreMount::ALL);
}

/// Every [`CoreMount`] of `plugin_id`, keyed by its host folder as Extism keys its map. The keys
/// are plain host paths, never prefixed: directories are created and host files named back
/// through them; the Extism key comes from [`to_extism_mount_key`] with
/// [`CoreMount::is_read_only`].
pub(crate) fn get_default_allowed_paths(
    location_info: &LocationInfo,
    plugin_id: &PluginId,
) -> HashMap<String, CoreMount> {
    CoreMount::ALL
        .into_iter()
        .map(|mount| {
            (
                mount
                    .host_root(location_info, plugin_id)
                    .to_string_lossy()
                    .into_owned(),
                mount,
            )
        })
        .collect()
}

/// The host file behind `plugin_path`, a plugin-side path such as `/cache/pack.zip`, held to
/// the mounts in `mounts`.
///
/// The only way from a plugin's spelling of a path to one the core opens, and closed by
/// default: anything but `<mount root>[/<relative path>]` is refused, a host path included,
/// since one would let a plugin have the core read or copy any file of the user's. `..`, roots
/// and drive or UNC prefixes in the relative part are refused before the filesystem is touched
/// (a UNC path alone would make Windows connect to the named server). The file must exist and
/// stay inside the mount after resolving symlinks and short names; the canonical path is
/// returned, so a symlink swapped in later does not redirect the core.
pub(crate) fn resolve_plugin_path(
    location_info: &LocationInfo,
    plugin_id: &PluginId,
    plugin_path: &str,
    mounts: MountSet,
) -> Result<PathBuf, PluginError> {
    let deny = || access_violation(plugin_id, plugin_path);

    let (mount, relative) = mounts
        .0
        .iter()
        .find_map(|mount| {
            plugin_path
                .strip_prefix(mount.plugin_root())
                .filter(|rest| rest.is_empty() || rest.starts_with('/'))
                .map(|rest| (*mount, rest.trim_start_matches('/')))
        })
        .ok_or_else(deny)?;

    let relative = Path::new(relative);
    let is_plain = relative.components().all(|component| match component {
        Component::CurDir => true,
        // `file:stream` names an NTFS alternate data stream, not the file it reads as.
        Component::Normal(name) => !(cfg!(windows) && name.to_string_lossy().contains(':')),
        Component::Prefix(_) | Component::RootDir | Component::ParentDir => false,
    });
    if !is_plain {
        return Err(deny());
    }

    let base = mount.host_root(location_info, plugin_id);
    let canonical_base = canonicalize_or_deny(plugin_id, &base)?;
    let canonical_host = canonicalize_or_deny(plugin_id, &base.join(relative))?;

    if !canonical_host.starts_with(&canonical_base) {
        return Err(deny());
    }

    Ok(canonical_host)
}

/// The plugin-side path of an existing host file under one of the plugin's mounts (`/cache`,
/// `/instances`); `None` for a file the plugin could not open.
pub(crate) fn host_path_to_plugin(
    location_info: &LocationInfo,
    plugin_id: &PluginId,
    host_path: &Path,
) -> Option<String> {
    let host_path = crate::shared::io::infra::canonicalize(host_path).ok()?;

    get_default_allowed_paths(location_info, plugin_id)
        .into_iter()
        .find_map(|(host_dir, mount)| {
            let base = crate::shared::io::infra::canonicalize(Path::new(&host_dir)).ok()?;
            let rest = host_path.strip_prefix(&base).ok()?;

            let mut plugin_path = mount.plugin_root().to_owned();
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

fn access_violation(id: &PluginId, path: impl AsRef<Path>) -> PluginError {
    PluginError::AccessViolation {
        plugin_id: id.clone(),
        path: path.as_ref().to_string_lossy().to_string(),
    }
}

/// Canonicalise or refuse. A path that cannot be resolved — it does not exist, or it is not
/// reachable — is never given the benefit of the doubt.
fn canonicalize_or_deny(id: &PluginId, path: &Path) -> Result<PathBuf, PluginError> {
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

/// How `run_command` reads a value the plugin wrote: `#<mount path>` resolves through
/// [`resolve_plugin_path`] into either mount, `/mnt/<drive>/…` is turned back into the host path
/// it spells, and anything else is kept as written.
///
/// Deliberately not a containment check, and the one place a plugin's string reaches the host
/// unresolved: an argument is opaque to the core (a flag, a URL, a path the program reads), so
/// there is nothing to hold it to. What fences `run_command` in is the program allowlist and the
/// mounted working directory, both checked on the result; this must not be reused where a path
/// is opened by the core itself.
fn command_value_to_host(
    id: &PluginId,
    value: &str,
    location_info: &LocationInfo,
) -> Result<String, PluginError> {
    if let Some(mount_path) = value.strip_prefix('#') {
        return resolve_plugin_path(location_info, id, mount_path, MountSet::ALL)
            .map(|path| path.to_string_lossy().into_owned());
    }

    if value.starts_with("/mnt/") {
        return Ok(from_wasi_path(value));
    }

    Ok(value.to_owned())
}

/// Refuse a network or device path before anything opens it, unless one of `trusted_roots` is
/// such a path too: merely resolving `\\server\share\…` makes Windows connect to that server and
/// offer it the user's credentials.
fn refuse_untrusted_share(
    id: &PluginId,
    path: &Path,
    trusted_roots: &[PathBuf],
) -> Result<(), PluginError> {
    if is_share_or_device(path) && !trusted_roots.iter().any(|root| is_share_or_device(root)) {
        return Err(access_violation(id, path));
    }
    Ok(())
}

fn is_share_or_device(path: &Path) -> bool {
    matches!(
        path.components().next(),
        Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), Prefix::UNC(..) | Prefix::VerbatimUNC(..) | Prefix::DeviceNS(_))
    )
}

/// Resolve `program` to an executable a plugin is allowed to start: one in a directory of
/// [`java_bin_dirs`], judged by its canonical directory, never by its file name.
///
/// A narrowing (no shell, no arbitrary native binary), not a containment boundary: the plugin
/// chooses the arguments, so `<allowed java> -jar #/cache/…` runs whatever it put there.
fn resolve_allowed_program(
    id: &PluginId,
    program: &str,
    location_info: &LocationInfo,
    allowed_program_dirs: &[PathBuf],
) -> Result<String, PluginError> {
    let resolved = PathBuf::from(command_value_to_host(id, program, location_info)?);
    refuse_untrusted_share(id, &resolved, allowed_program_dirs)?;

    // No fallback to the raw string: a `program` that does not resolve is refused instead of
    // being launched as written.
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
    id: &PluginId,
    current_dir: Option<&Path>,
    location_info: &LocationInfo,
) -> Result<PathBuf, PluginError> {
    let current_dir = current_dir.ok_or_else(|| access_violation(id, "<no current_dir>"))?;

    let resolved = PathBuf::from(command_value_to_host(
        id,
        &current_dir.to_string_lossy(),
        location_info,
    )?);

    let mount_roots: Vec<PathBuf> = CoreMount::ALL
        .into_iter()
        .map(|mount| mount.host_root(location_info, id))
        .collect();
    refuse_untrusted_share(id, &resolved, &mount_roots)?;

    let canonical = canonicalize_or_deny(id, &resolved)?;

    let is_mounted = mount_roots
        .iter()
        .filter_map(|root| crate::shared::io::infra::canonicalize(root).ok())
        .any(|root| canonical.starts_with(&root));

    if !is_mounted {
        return Err(access_violation(id, &canonical));
    }

    Ok(canonical)
}

pub(crate) fn plugin_command_to_host(
    id: &PluginId,
    command: &CommandDto,
    location_info: &LocationInfo,
    allowed_program_dirs: &[PathBuf],
) -> Result<SerializableCommand, PluginError> {
    let resolved_program =
        resolve_allowed_program(id, &command.program, location_info, allowed_program_dirs)?;

    let resolved_args: Vec<String> = command
        .args
        .iter()
        .map(|arg| command_value_to_host(id, arg, location_info))
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::{from_wasi_path, to_wasi_path};
    use crate::features::instance::InstanceId;
    use rstest::rstest;

    #[rstest]
    #[case::drive_with_backslash(
        r"D:\Documents\Minecraft\Test\simple\pack.toml",
        "/mnt/d/Documents/Minecraft/Test/simple/pack.toml"
    )]
    #[case::drive_with_slash("C:/Users/test/file.txt", "/mnt/c/Users/test/file.txt")]
    #[case::drive_root("D:\\", "/mnt/d")]
    #[case::already_wasi("/mnt/c/path/to/file", "/mnt/c/path/to/file")]
    #[case::linux_absolute("/home/user/file.txt", "/home/user/file.txt")]
    #[case::relative("relative/path/file.txt", "relative/path/file.txt")]
    #[case::lowercase_drive("d:/path/to/file", "/mnt/d/path/to/file")]
    #[case::trailing_slash(r"C:\Users\", "/mnt/c/Users")]
    #[case::double_slashes("C://path//to//file", "/mnt/c/path/to/file")]
    #[case::empty("", "")]
    #[case::colon_without_drive_letter("some:path", "some:path")]
    fn should_convert_to_wasi_path(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_wasi_path(input), expected);
    }

    // ── from_wasi_path tests ──

    #[cfg(windows)]
    #[rstest]
    #[case::drive_and_file(
        "/mnt/d/Documents/Minecraft/Test/simple/pack.toml",
        r"D:\Documents\Minecraft\Test\simple\pack.toml"
    )]
    #[case::lowercase_drive_only("/mnt/c", r"C:\")]
    #[case::uppercase_drive_only("/mnt/D", r"D:\")]
    fn should_convert_from_wasi_path(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(from_wasi_path(input), expected);
    }

    #[rstest]
    #[case::linux_path("/home/user/file.txt")]
    #[case::relative_path("relative/path/file.txt")]
    #[case::empty_path("")]
    fn should_leave_the_path_as_is_when_converting_from_wasi(#[case] path: &str) {
        assert_eq!(from_wasi_path(path), path);
    }

    #[test]
    #[cfg(windows)]
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
        let allowed = get_default_allowed_paths(&location_info, &PluginId::from_static(PLUGIN_ID));

        let cache_host = location_info
            .plugin_cache_dir(&PluginId::from_static(PLUGIN_ID))
            .to_string_lossy()
            .to_string();
        let instances_host = location_info.instances_dir().to_string_lossy().to_string();

        assert_eq!(
            to_extism_mount_key(&instances_host, allowed[&instances_host].is_read_only()),
            format!("ro:{instances_host}"),
            "the instance tree must reach the sandbox read-only"
        );
        assert_eq!(
            to_extism_mount_key(&cache_host, allowed[&cache_host].is_read_only()),
            cache_host,
            "the plugin cache stays read-write: it is the plugin's own scratch space"
        );
    }

    #[test]
    fn should_keep_the_read_only_prefix_out_of_the_default_mapping_when_it_is_built() {
        // The map itself feeds `create_dir_all`, so a `ro:`-prefixed key there would mean a
        // directory literally named `ro:...`.
        let location_info = LocationInfo::new(PathBuf::from("settings"), PathBuf::from("config"));

        assert!(
            get_default_allowed_paths(&location_info, &PluginId::from_static(PLUGIN_ID))
                .keys()
                .all(|host| !host.starts_with(READ_ONLY_MOUNT_PREFIX))
        );
    }

    // ── plugin_command_to_host: the run_command allowlist ──

    const PLUGIN_ID: &str = "packwiz";

    fn plugin_id() -> PluginId {
        PluginId::from_static(PLUGIN_ID)
    }

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

        let plugin_cache_dir = location_info.plugin_cache_dir(&plugin_id());
        std::fs::create_dir_all(&plugin_cache_dir).expect("plugin cache dir");
        std::fs::write(
            plugin_cache_dir.join("packwiz-installer-bootstrap.jar"),
            b"",
        )
        .expect("bootstrap jar");

        std::fs::create_dir_all(
            location_info.instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap()),
        )
        .expect("instance dir");

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
        // `ro:` narrows WASI only: `run_command` still has to reach into `/instances`.
        let layout = layout();

        let resolved = resolve_plugin_path(
            &layout.location_info,
            &plugin_id(),
            &format!("/instances/{INSTANCE_ID}"),
            MountSet::ALL,
        )
        .expect("an instance path must still resolve");

        assert_eq!(
            crate::shared::io::infra::canonicalize(&resolved).unwrap(),
            crate::shared::io::infra::canonicalize(
                layout
                    .location_info
                    .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
            )
            .unwrap()
        );
    }

    #[test]
    fn should_accept_a_program_when_it_is_a_host_managed_java_path() {
        let layout = layout();
        let command = packwiz_command(&layout.java_bin.to_string_lossy());

        let host_command = plugin_command_to_host(
            &plugin_id(),
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
            &plugin_id(),
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
                        &plugin_id(),
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
                &plugin_id(),
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
                &plugin_id(),
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
                    &plugin_id(),
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
                &plugin_id(),
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
                &plugin_id(),
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
            plugin_command_to_host(&plugin_id(), &command, &layout.location_info, &allowed).is_ok(),
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
                &plugin_id(),
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
                &plugin_id(),
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
                plugin_command_to_host(&plugin_id(), &command, &layout.location_info, &[]),
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
            &plugin_id(),
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
                .plugin_cache_dir(&plugin_id())
                .join("packwiz-installer-bootstrap.jar"),
        )
        .unwrap();

        let host = resolve_plugin_path(
            &layout.location_info,
            &plugin_id(),
            "/cache/packwiz-installer-bootstrap.jar",
            MountSet::CACHE,
        )
        .expect("a file in the plugin's own cache");

        assert_eq!(host, expected);
    }

    fn assert_refused(layout: &Layout, path: &str, mounts: MountSet) {
        assert!(
            matches!(
                resolve_plugin_path(&layout.location_info, &plugin_id(), path, mounts),
                Err(PluginError::AccessViolation { .. })
            ),
            "{path} must be refused"
        );
    }

    #[test]
    fn should_refuse_a_path_when_it_is_outside_the_plugin_cache() {
        let layout = layout();
        let instance_dir = layout
            .location_info
            .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap());
        let other_mount = format!("/instances/{INSTANCE_ID}");

        for path in [
            // A bare host path.
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
            assert_refused(&layout, path, MountSet::CACHE);
        }
    }

    #[test]
    fn should_resolve_into_either_mount_when_both_are_allowed() {
        let layout = layout();

        for path in [
            "/cache".to_owned(),
            "/cache/packwiz-installer-bootstrap.jar".to_owned(),
            "/cache/./packwiz-installer-bootstrap.jar".to_owned(),
            "/instances".to_owned(),
            format!("/instances/{INSTANCE_ID}"),
            format!("/instances/{INSTANCE_ID}/"),
        ] {
            resolve_plugin_path(&layout.location_info, &plugin_id(), &path, MountSet::ALL)
                .unwrap_or_else(|err| panic!("{path} must resolve: {err}"));
        }
    }

    #[test]
    fn should_refuse_a_path_when_it_leaves_the_mount() {
        let layout = layout();

        for path in [
            "/cache/..",
            "/cache/../packwiz",
            "/cache/sub/../../x",
            "/instances/..",
            "/instances/../plugins",
            // Another plugin's cache sits next to this one.
            "/cache/../another-plugin",
            "/cache//../x",
            "",
            "/",
            "cache/packwiz-installer-bootstrap.jar",
            "/Cache/packwiz-installer-bootstrap.jar",
            "/instancesx",
        ] {
            assert_refused(&layout, path, MountSet::ALL);
        }
    }

    #[test]
    fn should_refuse_a_host_path_when_it_is_absolute_in_any_spelling() {
        let layout = layout();
        let config = layout
            .location_info
            .config_dir()
            .to_string_lossy()
            .into_owned();

        for path in [
            config.as_str(),
            "/etc/passwd",
            "/mnt/c/Windows/System32/cmd.exe",
            r"C:\Windows\System32\cmd.exe",
            "C:/Windows",
            r"\\server\share\file",
            r"\\?\C:\Windows",
            r"\\?\UNC\server\share",
            r"\\.\pipe\name",
            r"C:\PROGRA~1",
        ] {
            assert_refused(&layout, path, MountSet::ALL);
        }
    }

    #[test]
    fn should_refuse_a_prefix_or_root_when_it_is_inside_the_relative_part() {
        let layout = layout();

        for path in [
            r"/cache/C:\Windows",
            "/cache/C:/Windows",
            r"/cache/\\server\share",
            r"/cache/\\?\C:\Windows",
            "/cache/packwiz-installer-bootstrap.jar:stream",
        ] {
            let result =
                resolve_plugin_path(&layout.location_info, &plugin_id(), path, MountSet::ALL);
            if cfg!(windows) {
                assert!(
                    matches!(result, Err(PluginError::AccessViolation { .. })),
                    "{path} must be refused"
                );
            } else {
                // Backslashes and colons are plain file name characters off Windows: whatever
                // such a name resolves to has to stay in the mount.
                match result {
                    Ok(host) => assert!(
                        host.starts_with(
                            crate::shared::io::infra::canonicalize(
                                layout.location_info.plugin_cache_dir(&plugin_id())
                            )
                            .unwrap()
                        ),
                        "{path} resolved outside the mount: {host:?}"
                    ),
                    Err(err) => assert!(
                        matches!(err, PluginError::AccessViolation { .. }),
                        "{path} failed with something other than a refusal: {err:?}"
                    ),
                }
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn should_refuse_a_traversal_when_it_is_spelled_with_backslashes() {
        let layout = layout();

        for path in [
            r"/cache\..\..\config",
            r"/cache/sub\..\..\x",
            r"/instances\..",
        ] {
            assert_refused(&layout, path, MountSet::ALL);
        }
    }

    #[test]
    fn should_refuse_the_instances_mount_when_only_the_cache_is_allowed() {
        let layout = layout();
        std::fs::write(
            layout
                .location_info
                .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
                .join("pack.toml"),
            b"",
        )
        .unwrap();

        for path in [
            "/instances".to_owned(),
            format!("/instances/{INSTANCE_ID}/pack.toml"),
        ] {
            assert_refused(&layout, &path, MountSet::CACHE);
        }
    }

    #[test]
    #[cfg(unix)]
    fn should_refuse_a_symlink_when_it_points_out_of_the_mount() {
        let layout = layout();
        let cache = layout.location_info.plugin_cache_dir(&plugin_id());
        std::os::unix::fs::symlink(layout.location_info.config_dir(), cache.join("escape"))
            .unwrap();

        assert_refused(&layout, "/cache/escape", MountSet::ALL);
    }

    #[test]
    fn should_refuse_a_hash_argument_when_it_leaves_the_mounts() {
        let layout = layout();

        for arg in [
            "#/cache/../../config",
            "#C:/Windows",
            "#/etc",
            r"#\\server\share",
        ] {
            let mut command = packwiz_command(&layout.java_bin.to_string_lossy());
            command.args.push(arg.to_owned());

            assert!(
                matches!(
                    plugin_command_to_host(
                        &plugin_id(),
                        &command,
                        &layout.location_info,
                        &layout.allowed()
                    ),
                    Err(PluginError::AccessViolation { .. })
                ),
                "{arg} must be refused"
            );
        }
    }

    #[test]
    fn should_pass_a_plain_argument_through_when_it_is_not_a_mount_path() {
        let layout = layout();
        let mut command = packwiz_command(&layout.java_bin.to_string_lossy());
        command.args.push("/etc/hosts".to_owned());

        let host_command = plugin_command_to_host(
            &plugin_id(),
            &command,
            &layout.location_info,
            &layout.allowed(),
        )
        .expect("allowed");

        assert_eq!(host_command.args.last().unwrap(), "/etc/hosts");
    }

    #[test]
    #[cfg(windows)]
    fn should_refuse_a_program_when_it_is_on_a_network_share() {
        let layout = layout();

        for program in [r"\\server\share\java.exe", r"\\?\UNC\server\share\java.exe"] {
            let command = packwiz_command(program);
            assert!(
                matches!(
                    plugin_command_to_host(
                        &plugin_id(),
                        &command,
                        &layout.location_info,
                        &layout.allowed()
                    ),
                    Err(PluginError::AccessViolation { .. })
                ),
                "{program} must be refused"
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn should_refuse_a_current_dir_when_it_is_on_a_network_share() {
        let layout = layout();
        let mut command = packwiz_command(&layout.java_bin.to_string_lossy());
        command.current_dir = Some(PathBuf::from(r"\\server\share\instances"));

        assert!(matches!(
            plugin_command_to_host(
                &plugin_id(),
                &command,
                &layout.location_info,
                &layout.allowed()
            ),
            Err(PluginError::AccessViolation { .. })
        ));
    }

    #[test]
    fn should_accept_a_current_dir_when_it_is_a_host_path_inside_a_mount() {
        let layout = layout();
        let mut command = packwiz_command(&layout.java_bin.to_string_lossy());
        command.current_dir = Some(
            layout
                .location_info
                .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap()),
        );

        let host_command = plugin_command_to_host(
            &plugin_id(),
            &command,
            &layout.location_info,
            &layout.allowed(),
        )
        .expect("a host path inside `/instances` is a mounted working directory");

        assert!(host_command.current_dir.unwrap().ends_with(INSTANCE_ID));
    }

    #[test]
    fn should_name_a_host_file_through_the_plugin_mount_when_the_plugin_can_read_it() {
        let layout = layout();
        let pack = layout
            .location_info
            .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .join("pack.toml");
        std::fs::write(&pack, b"").unwrap();

        assert_eq!(
            host_path_to_plugin(&layout.location_info, &plugin_id(), &pack).as_deref(),
            Some(format!("/instances/{INSTANCE_ID}/pack.toml").as_str())
        );
        assert_eq!(
            host_path_to_plugin(
                &layout.location_info,
                &plugin_id(),
                &layout
                    .location_info
                    .plugin_cache_dir(&plugin_id())
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
            host_path_to_plugin(&layout.location_info, &plugin_id(), &layout.java_bin),
            None
        );
        assert_eq!(
            host_path_to_plugin(
                &layout.location_info,
                &PluginId::from_static("another-plugin"),
                &layout
                    .location_info
                    .plugin_cache_dir(&plugin_id())
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
