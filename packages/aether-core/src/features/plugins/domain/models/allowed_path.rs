use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

use crate::features::{instance::PluginId, settings::LocationInfo};

/// Where a plugin sees its own cache directory.
pub(crate) const PLUGIN_CACHE_MOUNT: &str = "/cache";

/// Where a plugin sees the instance tree.
pub(crate) const PLUGIN_INSTANCES_MOUNT: &str = "/instances";

/// A directory the core mounts into every plugin's sandbox: the one description of those mounts,
/// from which the path resolution, the Extism map, the read-only marking and the names a folder
/// of the user's must stay clear of are all derived.
///
/// A folder of the user's must not take their place or sit over or under them: the core's
/// guarantees about them (`/instances` read-only, `/cache` the plugin's own) hold only while
/// nothing else answers for those names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoreMount {
    /// The plugin's own scratch space, read-write.
    Cache,
    /// The tree of all instances, read-only to the sandbox.
    Instances,
}

impl CoreMount {
    pub(crate) const ALL: [Self; 2] = [Self::Cache, Self::Instances];

    /// Where the plugin sees the mount.
    pub(crate) fn plugin_root(self) -> &'static str {
        match self {
            Self::Cache => PLUGIN_CACHE_MOUNT,
            Self::Instances => PLUGIN_INSTANCES_MOUNT,
        }
    }

    /// The host folder behind the mount, for `plugin_id`.
    pub(crate) fn host_root(self, location_info: &LocationInfo, plugin_id: &PluginId) -> PathBuf {
        match self {
            Self::Cache => location_info.plugin_cache_dir(plugin_id),
            Self::Instances => location_info.instances_dir(),
        }
    }

    /// Whether the WASM sandbox may only read the mount.
    ///
    /// Every write into an instance goes through `write_instance_file`, the single checked path,
    /// which also guards `.metadata`. `/cache` stays read-write: it is the plugin's own scratch
    /// space.
    pub(crate) fn is_read_only(self) -> bool {
        match self {
            Self::Cache => false,
            Self::Instances => true,
        }
    }
}

/// A folder of the user's that a plugin may open, and the path it sees it under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedPath {
    /// Absolute path on the host.
    pub host: String,
    /// Absolute path inside the plugin sandbox, `/`-separated.
    pub plugin: String,
    /// The plugin may only read the folder. The default: a plugin gets to write only where the
    /// user explicitly said so.
    pub read_only: bool,
}

/// Why a folder cannot be given to a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedPathRejection {
    /// The host path is relative: it would resolve against the launcher's working directory.
    HostNotAbsolute,
    /// The host folder does not exist, or is not a folder.
    HostNotAFolder,
    /// A whole drive or the filesystem root.
    HostIsRoot,
    /// The plugin path does not start with `/`.
    PluginNotAbsolute,
    /// The plugin path has empty, `.` or `..` segments.
    PluginNotNormalized,
    /// The plugin path is `/` itself, which would hide every other mount.
    PluginIsRoot,
    /// The plugin path is, contains or lies inside the plugin root of a [`CoreMount`].
    PluginReserved,
    /// The host folder is, contains or lies inside one of [`core_folders`].
    HostOverlapsCore,
}

impl std::fmt::Display for AllowedPathRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::HostNotAbsolute => "the host path is not absolute",
            Self::HostNotAFolder => "the host path is not an existing folder",
            Self::HostIsRoot => "a whole drive is never given to a plugin",
            Self::PluginNotAbsolute => "the plugin path must start with `/`",
            Self::PluginNotNormalized => {
                "the plugin path must not contain empty, `.` or `..` parts"
            }
            Self::PluginIsRoot => "the plugin path must not be `/`",
            Self::PluginReserved => "the plugin path overlaps `/cache` or `/instances`",
            Self::HostOverlapsCore => {
                "the host folder overlaps a folder the launcher keeps for itself: the instances, \
                 the plugins, the caches, the game files or the settings"
            }
        })
    }
}

impl AllowedPath {
    /// `folder` mounted read-only where the plugin will look for it: under its WASI name, the
    /// one the core hands the plugin a file of that folder by.
    pub fn read_only_folder(folder: &Path) -> Self {
        let host = folder.to_string_lossy().into_owned();

        Self {
            plugin: to_wasi_path(&host),
            host,
            read_only: true,
        }
    }

    /// The plugin path written the one way it is stored and compared: a host-style path the
    /// user typed or picked (`D:\packs`) becomes the plugin's name for it (`/mnt/d/packs`).
    #[must_use]
    pub fn normalized(mut self) -> Self {
        self.plugin = to_wasi_path(&self.plugin);
        self
    }

    /// Checks both sides of the mapping, not the existence of the host folder: a folder that is
    /// gone is dropped from the settings rather than refused.
    pub fn validate(&self) -> Result<(), AllowedPathRejection> {
        let host = Path::new(&self.host);

        if !host.is_absolute() {
            return Err(AllowedPathRejection::HostNotAbsolute);
        }

        // A whole drive holds every folder of the user's and the launcher's own: no overlap
        // check could keep the plugin out of them.
        if is_filesystem_root(host) {
            return Err(AllowedPathRejection::HostIsRoot);
        }

        validate_plugin_path(&self.plugin)
    }

    /// [`Self::validate`], and the host folder must not overlap any of [`core_folders`]: it
    /// would replace the core's mount in Extism's map, or sidestep the read-only `/instances`.
    pub fn validate_against(&self, core_folders: &[PathBuf]) -> Result<(), AllowedPathRejection> {
        self.validate()?;

        if overlaps_core_folder(&self.host, core_folders) {
            return Err(AllowedPathRejection::HostOverlapsCore);
        }

        Ok(())
    }

    /// Whether the plugin can open `file` (a host path) by the name the core gives it, its WASI
    /// name. Only a mount under its own WASI name answers for that name, so a folder mapped
    /// elsewhere does not count.
    pub fn exposes(&self, file: &Path) -> bool {
        exposes(&self.host, &self.plugin, file)
    }
}

fn exposes(host: &str, plugin: &str, file: &Path) -> bool {
    plugin == to_wasi_path(host) && is_within(&to_wasi_path(&file.to_string_lossy()), plugin)
}

/// The user's folders as an instance gets them, keyed like the map Extism builds (the later
/// entry winning). Two lists that mount the same are equal whatever their order or repeats, so
/// comparing with the settings tells exactly whether a restart would change what the plugin sees.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderMounts(BTreeMap<(String, bool), String>);

impl FolderMounts {
    pub fn of<'a>(paths: impl IntoIterator<Item = &'a AllowedPath>) -> Self {
        Self(
            paths
                .into_iter()
                .map(|path| ((path.host.clone(), path.read_only), path.plugin.clone()))
                .collect(),
        )
    }

    /// [`AllowedPath::exposes`] for any of these folders.
    pub fn exposes(&self, file: &Path) -> bool {
        self.0
            .iter()
            .any(|((host, _), plugin)| exposes(host, plugin, file))
    }
}

fn validate_plugin_path(plugin: &str) -> Result<(), AllowedPathRejection> {
    let Some(rest) = plugin.strip_prefix('/') else {
        return Err(AllowedPathRejection::PluginNotAbsolute);
    };

    if rest.is_empty() {
        return Err(AllowedPathRejection::PluginIsRoot);
    }

    // `..` here would not escape on the host — the mount is a capability — but it would make
    // the reserved-mount check below compare names that do not mean what they say.
    if rest
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(AllowedPathRejection::PluginNotNormalized);
    }

    if CoreMount::ALL
        .into_iter()
        .map(CoreMount::plugin_root)
        .any(|mount| is_within(plugin, mount) || is_within(mount, plugin))
    {
        return Err(AllowedPathRejection::PluginReserved);
    }

    Ok(())
}

/// The host folders a folder of the user's must stay clear of: those the core mounts for the
/// plugin itself, and everything the core runs or trusts.
///
/// - `plugins/` holds each plugin's binary, `settings.toml` and trust record: a plugin that could
///   write there would grant itself folders and hosts, or replace another plugin.
/// - `cache/` holds the Java runtimes the core starts (`run_command` allows what is under their
///   `bin`), the compiled wasm it loads, and the caches of the other plugins.
/// - `.minecraft/` holds the libraries, natives and game versions the core puts on the game's
///   class path and launches.
/// - `data/` holds the app database, and the settings folder the settings and the credentials.
pub(crate) fn core_folders(location_info: &LocationInfo, plugin_id: &PluginId) -> Vec<PathBuf> {
    // The plugin's cache is within `cache/`, but the one folder there it is given itself.
    CoreMount::ALL
        .into_iter()
        .map(|mount| mount.host_root(location_info, plugin_id))
        .chain([
            location_info.cache_dir(),
            location_info.plugins_dir(),
            location_info.metadata_dir(),
            location_info.data_dir(),
            location_info.settings_dir().to_path_buf(),
        ])
        .collect()
}

/// The folders of a plugin's settings that its instance is built with, and those it is not,
/// each with why. The settings file is not only written by the edits that check a folder: it
/// may predate a check, or have been changed by hand, so what is stored is not all mounted.
/// Whatever asks what a plugin can open goes by the first list, never by the settings as saved.
pub(crate) fn partition_mountable<'a>(
    paths: &'a [AllowedPath],
    core_folders: &[PathBuf],
) -> (
    Vec<&'a AllowedPath>,
    Vec<(&'a AllowedPath, AllowedPathRejection)>,
) {
    let mut mountable = Vec::new();
    let mut refused = Vec::new();

    for path in paths {
        match path.validate_against(core_folders) {
            Ok(()) => mountable.push(path),
            Err(reason) => refused.push((path, reason)),
        }
    }

    (mountable, refused)
}

/// Whether `host` (a stored folder) overlaps a folder of the core. For the loader, which drops
/// such a folder from settings written before the check existed rather than fail the plugin.
pub(super) fn overlaps_core_folder(host: &str, core_folders: &[PathBuf]) -> bool {
    let host = comparable(Path::new(host));

    core_folders.iter().any(|core| {
        let core = comparable(core);
        is_within(&host, &core) || is_within(&core, &host)
    })
}

/// Whether `path` is a whole drive or the filesystem root, however it is written to lead
/// there: `D:\x\..` is `D:\`, and `Path::parent` alone does not see it. Resolved against the
/// disk when the path exists (a link to the root is the root); otherwise `..` is taken out
/// without touching the disk.
pub(crate) fn is_filesystem_root(path: &Path) -> bool {
    dunce::canonicalize(path)
        .unwrap_or_else(|_| lexically_normalized(path))
        .parent()
        .is_none()
}

/// `path` without `.` and `..` segments, by the names alone. A `..` at the root stays there, as
/// it does on disk.
fn lexically_normalized(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) {
                    normalized.pop();
                }
            }
            other => normalized.push(other),
        }
    }

    normalized
}

/// A host path in the one form two paths are compared in: symlinks and `..` resolved as far as
/// the path exists (the rest is taken as written), `/`-separated, and without case on Windows,
/// whose file system has none.
fn comparable(path: &Path) -> String {
    let resolved = resolve_existing(path);
    let text = resolved.to_string_lossy();

    if cfg!(windows) {
        text.replace('\\', "/").trim_end_matches('/').to_lowercase()
    } else {
        text.trim_end_matches('/').to_owned()
    }
}

fn resolve_existing(path: &Path) -> PathBuf {
    if let Ok(resolved) = dunce::canonicalize(path) {
        return resolved;
    }

    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => resolve_existing(parent).join(name),
        _ => path.to_path_buf(),
    }
}

/// `path` is `base` or lies under it, by whole segments.
fn is_within(path: &str, base: &str) -> bool {
    path.strip_prefix(base)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// Convert a Windows absolute path (e.g. `D:\path\to\file`) to a WASI-compatible
/// path (`/mnt/d/path/to/file`). Non-Windows paths are returned as-is with `/` normalization.
pub(crate) fn to_wasi_path(input: &str) -> String {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn absolute(path: &str) -> String {
        if cfg!(windows) {
            format!(r"C:\{}", path.replace('/', r"\"))
        } else {
            format!("/{path}")
        }
    }

    fn mapping(plugin: &str) -> AllowedPath {
        AllowedPath {
            host: absolute("packs"),
            plugin: plugin.to_owned(),
            read_only: true,
        }
    }

    #[test]
    fn should_accept_a_folder_when_it_is_mounted_under_its_own_name() {
        assert_eq!(mapping("/mnt/c/packs").validate(), Ok(()));
        assert_eq!(mapping("/packs").validate(), Ok(()));
        assert_eq!(
            mapping("/cached").validate(),
            Ok(()),
            "a lookalike is no overlap"
        );
    }

    #[test]
    fn should_refuse_a_host_path_when_it_is_relative() {
        // It would be opened relative to the launcher's working directory.
        for host in ["packs", "../..", "ro:packs"] {
            let path = AllowedPath {
                host: host.to_owned(),
                ..mapping("/packs")
            };
            assert_eq!(
                path.validate(),
                Err(AllowedPathRejection::HostNotAbsolute),
                "{host}"
            );
        }
    }

    #[test]
    fn should_refuse_plugin_paths_when_they_lie_over_or_under_the_core_mounts() {
        for plugin in ["/cache", "/cache/x", "/instances", "/instances/a/b"] {
            assert_eq!(
                mapping(plugin).validate(),
                Err(AllowedPathRejection::PluginReserved),
                "{plugin}"
            );
        }
    }

    fn host_mapping(host: &Path, plugin: &str) -> AllowedPath {
        AllowedPath {
            host: host.to_string_lossy().into_owned(),
            plugin: plugin.to_owned(),
            read_only: false,
        }
    }

    #[test]
    fn should_refuse_a_host_folder_when_it_overlaps_a_core_folder() {
        let root = tempfile::tempdir().unwrap();
        let cache = root.path().join("cache").join("plugins").join("p");
        let instances = root.path().join("instances");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::create_dir_all(instances.join("a")).unwrap();
        let core = [cache.clone(), instances.clone()];

        for host in [
            cache.clone(),
            instances.clone(),
            // Inside, and containing, either of them.
            instances.join("a"),
            cache.join("sub"),
            root.path().to_path_buf(),
            root.path().join("cache"),
            // The same folder reached another way.
            instances.join("a").join(".."),
            PathBuf::from(format!("{}/", instances.to_string_lossy())),
        ] {
            assert_eq!(
                host_mapping(&host, "/x").validate_against(&core),
                Err(AllowedPathRejection::HostOverlapsCore),
                "{}",
                host.display()
            );
        }
    }

    #[test]
    fn should_refuse_a_host_folder_when_it_overlaps_the_plugins_folder() {
        let root = tempfile::tempdir().unwrap();
        let location_info =
            LocationInfo::new(root.path().join("settings"), root.path().join("config"));
        std::fs::create_dir_all(location_info.plugin_dir(&PluginId::from_static("other"))).unwrap();
        let core = core_folders(&location_info, &PluginId::from_static("self"));

        for host in [
            location_info.plugins_dir(),
            location_info.plugin_dir(&PluginId::from_static("self")),
            location_info.plugin_dir(&PluginId::from_static("other")),
            location_info.config_dir().to_path_buf(),
        ] {
            assert_eq!(
                host_mapping(&host, "/x").validate_against(&core),
                Err(AllowedPathRejection::HostOverlapsCore),
                "{}",
                host.display()
            );
        }
    }

    #[test]
    fn should_refuse_a_host_folder_when_it_overlaps_what_the_core_runs_or_trusts() {
        let root = tempfile::tempdir().unwrap();
        let location_info =
            LocationInfo::new(root.path().join("settings"), root.path().join("config"));
        let core = core_folders(&location_info, &PluginId::from_static("self"));

        for host in [
            // The Java runtimes, whose `bin` the core lets a plugin start.
            location_info.java_dir(),
            location_info.java_dir().join("jre-21"),
            location_info.wasm_cache_dir(),
            location_info.cache_dir(),
            // The cache of another plugin.
            location_info.plugin_cache_dir(&PluginId::from_static("other")),
            location_info.cache_dir().join("plugins"),
            location_info.libraries_dir(),
            location_info.natives_dir(),
            location_info.version_dir("1.21"),
            location_info.assets_dir(),
            location_info.metadata_dir(),
            location_info.data_dir(),
            // The settings and the credentials.
            location_info.settings_dir().to_path_buf(),
            location_info.settings_dir().join("credentials.json"),
        ] {
            assert_eq!(
                host_mapping(&host, "/x").validate_against(&core),
                Err(AllowedPathRejection::HostOverlapsCore),
                "{}",
                host.display()
            );
        }
    }

    #[test]
    fn should_accept_a_host_folder_when_it_is_beside_the_launcher_folders() {
        let root = tempfile::tempdir().unwrap();
        let location_info =
            LocationInfo::new(root.path().join("settings"), root.path().join("config"));
        let core = core_folders(&location_info, &PluginId::from_static("self"));

        for host in ["packs", "config-backup", "cached"] {
            assert_eq!(
                host_mapping(&root.path().join(host), "/x").validate_against(&core),
                Ok(()),
                "{host}"
            );
        }
    }

    #[test]
    fn should_refuse_a_host_when_it_is_a_whole_drive() {
        let roots: &[&str] = if cfg!(windows) {
            &[r"C:\", "C:/", r"\\server\share\"]
        } else {
            &["/"]
        };

        for host in roots {
            assert_eq!(
                host_mapping(Path::new(host), "/x").validate(),
                Err(AllowedPathRejection::HostIsRoot),
                "{host}"
            );
        }
    }

    #[test]
    fn should_refuse_a_host_when_dot_dot_leads_to_a_whole_drive() {
        let hosts: &[&str] = if cfg!(windows) {
            &[r"C:\x\..", r"C:\x\y\..\..", r"C:\.", r"C:\..", "C:/x/.."]
        } else {
            &["/x/..", "/x/y/../..", "/.", "/.."]
        };

        for host in hosts {
            assert_eq!(
                host_mapping(Path::new(host), "/x").validate(),
                Err(AllowedPathRejection::HostIsRoot),
                "{host}"
            );
        }
    }

    #[test]
    fn should_accept_a_host_when_dot_dot_stays_below_the_root() {
        let host = absolute("x/../packs");

        assert_eq!(host_mapping(Path::new(&host), "/x").validate(), Ok(()));
    }

    #[test]
    fn should_accept_a_host_folder_when_it_is_clear_of_the_core_folders() {
        let root = tempfile::tempdir().unwrap();
        let core = [root.path().join("cache"), root.path().join("instances")];
        std::fs::create_dir_all(root.path().join("packs")).unwrap();

        for host in ["packs", "instances2", "cached"] {
            assert_eq!(
                host_mapping(&root.path().join(host), "/x").validate_against(&core),
                Ok(()),
                "{host}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn should_compare_host_folders_without_case_when_the_host_is_windows() {
        let root = tempfile::tempdir().unwrap();
        let instances = root.path().join("instances");
        std::fs::create_dir_all(&instances).unwrap();

        let shouting = instances.to_string_lossy().to_uppercase();
        assert_eq!(
            host_mapping(Path::new(&shouting), "/x")
                .validate_against(std::slice::from_ref(&instances)),
            Err(AllowedPathRejection::HostOverlapsCore)
        );

        // Not created yet: compared as written.
        let missing = root.path().join("Later");
        let missing_upper = root.path().join("LATER").join("deep");
        assert_eq!(
            host_mapping(&missing_upper, "/x").validate_against(&[missing]),
            Err(AllowedPathRejection::HostOverlapsCore)
        );
    }

    #[test]
    fn should_refuse_a_plugin_path_when_it_is_malformed() {
        assert_eq!(
            mapping("packs").validate(),
            Err(AllowedPathRejection::PluginNotAbsolute)
        );
        assert_eq!(
            mapping("/").validate(),
            Err(AllowedPathRejection::PluginIsRoot)
        );
        for plugin in ["/packs/../instances", "/./cache", "/a//b", "/a/"] {
            assert_eq!(
                mapping(plugin).validate(),
                Err(AllowedPathRejection::PluginNotNormalized),
                "{plugin}"
            );
        }
    }

    #[test]
    fn should_normalize_a_plugin_path_when_it_is_host_style() {
        let path = AllowedPath {
            plugin: r"D:\packs\\simple\".to_owned(),
            ..mapping("")
        }
        .normalized();

        assert_eq!(path.plugin, "/mnt/d/packs/simple");
        assert_eq!(path.validate(), Ok(()));
    }

    #[test]
    fn should_mount_a_picked_folder_read_only_under_its_wasi_name_when_given_read_only_access() {
        let path = AllowedPath::read_only_folder(Path::new(r"D:\packs\simple"));

        assert_eq!(path.host, r"D:\packs\simple");
        assert_eq!(path.plugin, "/mnt/d/packs/simple");
        assert!(path.read_only);
    }

    #[test]
    fn should_leave_out_a_folder_when_it_cannot_be_mounted() {
        let root = tempfile::tempdir().unwrap();
        let cache = root.path().join("cache");
        let packs = AllowedPath::read_only_folder(&root.path().join("packs"));
        let in_core = AllowedPath::read_only_folder(&cache.join("sub"));
        let relative = AllowedPath {
            host: "packs".to_owned(),
            ..packs.clone()
        };

        let paths = [in_core.clone(), packs.clone(), relative.clone()];
        let (mountable, refused) = partition_mountable(&paths, &[cache]);

        assert_eq!(mountable, [&packs]);
        assert_eq!(
            refused,
            [
                (&in_core, AllowedPathRejection::HostOverlapsCore),
                (&relative, AllowedPathRejection::HostNotAbsolute),
            ]
        );
    }

    #[test]
    fn should_mount_the_same_when_the_order_or_repeats_of_entries_differ() {
        let packs = mapping("/packs");
        let other = AllowedPath {
            host: absolute("other"),
            ..mapping("/other")
        };

        assert_eq!(
            FolderMounts::of(&[packs.clone(), other.clone()]),
            FolderMounts::of(&[other.clone(), packs.clone(), packs.clone()])
        );
        assert_ne!(
            FolderMounts::of(std::slice::from_ref(&packs)),
            FolderMounts::of(&[packs, other])
        );
    }

    #[test]
    fn should_tell_mounts_apart_when_access_or_the_entry_extism_keeps_differs() {
        let read_only = mapping("/packs");
        let writable = AllowedPath {
            read_only: false,
            ..read_only.clone()
        };
        let moved = mapping("/moved");

        assert_ne!(
            FolderMounts::of(std::slice::from_ref(&read_only)),
            FolderMounts::of(std::slice::from_ref(&writable))
        );
        // One host folder with one access is one mount: the later entry is the one mounted.
        assert_eq!(
            FolderMounts::of(&[read_only.clone(), moved.clone()]),
            FolderMounts::of(std::slice::from_ref(&moved))
        );
        assert_ne!(
            FolderMounts::of(&[read_only.clone(), moved]),
            FolderMounts::of(&[mapping("/moved"), read_only])
        );
    }

    #[test]
    fn should_expose_only_files_when_they_are_under_a_mount_of_their_own_name() {
        let folder = AllowedPath::read_only_folder(Path::new(r"D:\packs"));

        assert!(folder.exposes(&PathBuf::from(r"D:\packs\simple\pack.toml")));
        assert!(folder.exposes(&PathBuf::from(r"d:/packs/pack.toml")));
        assert!(!folder.exposes(&PathBuf::from(r"D:\packsx\pack.toml")));
        assert!(!folder.exposes(&PathBuf::from(r"E:\packs\pack.toml")));

        let renamed = AllowedPath {
            plugin: "/packs".to_owned(),
            ..folder
        };
        assert!(
            !renamed.exposes(&PathBuf::from(r"D:\packs\pack.toml")),
            "the plugin is handed `/mnt/d/packs/pack.toml`, which this mount does not answer"
        );
    }

    #[test]
    fn should_expose_a_file_through_mounts_when_one_of_them_exposes_it() {
        let packs = AllowedPath::read_only_folder(Path::new(r"D:\packs"));
        let other = AllowedPath::read_only_folder(Path::new(r"D:\other"));
        let file = PathBuf::from(r"D:\packs\simple\pack.toml");

        assert!(FolderMounts::of(&[other.clone(), packs]).exposes(&file));
        assert!(!FolderMounts::of(&[other]).exposes(&file));
        assert!(!FolderMounts::default().exposes(&file));
    }
}
