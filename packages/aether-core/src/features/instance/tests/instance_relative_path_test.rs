//! Path checks behind `write_instance_file` / `read_instance_file`.

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::features::instance::domain::{InstanceError, InstanceId};
use crate::features::instance::infra::{
    check_plugin_write_allowed, resolve_instance_file_path, validate_relative_path,
};
use crate::test_support::{link_dir, link_file};

const INSTANCE_ID: &str = "inst-1";

fn setup() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let instance_dir = temp_dir.path().join("instances").join(INSTANCE_ID);
    std::fs::create_dir_all(&instance_dir).expect("Failed to create instance dir");
    (temp_dir, instance_dir)
}

fn resolve(instance_dir: &Path, relative_path: &str) -> Result<PathBuf, InstanceError> {
    resolve_instance_file_path(
        &InstanceId::try_from(INSTANCE_ID).unwrap(),
        instance_dir,
        relative_path,
    )
}

// ── validate_relative_path ──

#[test]
fn should_accept_a_path_when_it_is_a_plain_relative_one() {
    let path = validate_relative_path("mods/example.jar").expect("plain path should be accepted");
    assert_eq!(path, PathBuf::from("mods").join("example.jar"));
}

#[test]
fn should_accept_a_path_when_it_is_a_single_file_name() {
    assert_eq!(
        validate_relative_path("pack.toml").expect("file name should be accepted"),
        PathBuf::from("pack.toml")
    );
}

#[test]
fn should_accept_a_path_when_it_uses_backslashes_as_separators() {
    assert_eq!(
        validate_relative_path(r"config\mod\settings.json").expect("should be accepted"),
        PathBuf::from("config").join("mod").join("settings.json")
    );
}

#[test]
fn should_reject_a_path_when_it_has_parent_directory_segments() {
    for path in [
        "../evil.txt",
        "mods/../../evil.txt",
        "mods/../evil.txt",
        r"mods\..\..\evil.txt",
        "..",
    ] {
        assert!(
            matches!(
                validate_relative_path(path),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "`{path}` must be rejected"
        );
    }
}

#[test]
fn should_reject_a_path_when_it_has_current_directory_segments() {
    for path in ["./pack.toml", "mods/./example.jar", "."] {
        assert!(
            matches!(
                validate_relative_path(path),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "`{path}` must be rejected"
        );
    }
}

#[test]
fn should_reject_a_path_when_it_is_absolute() {
    for path in [
        "/etc/passwd",
        "/instances/inst-1/pack.toml",
        r"\Windows\System32\evil.dll",
        "C:/Windows/System32/evil.dll",
        r"C:\Windows\System32\evil.dll",
        r"\\server\share\evil.dll",
        r"\\?\C:\evil.dll",
    ] {
        assert!(
            matches!(
                validate_relative_path(path),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "`{path}` must be rejected"
        );
    }
}

#[test]
fn should_reject_a_path_when_it_is_wasi_style() {
    // Plugins speak `/instances/…` and `/mnt/d/…` to WASI, and
    // neither form may be accepted here.
    for path in ["/instances/inst-1/mods/a.jar", "/mnt/d/instances/a.jar"] {
        assert!(
            matches!(
                validate_relative_path(path),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "WASI path `{path}` must be rejected"
        );
    }
}

#[test]
fn should_reject_a_path_when_it_is_empty_or_only_separators() {
    for path in ["", "/", r"\", "mods//a.jar", "mods/", r"mods\\a.jar"] {
        assert!(
            matches!(
                validate_relative_path(path),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "`{path}` must be rejected"
        );
    }
}

/// A first segment spelled as a drive is absolute on Windows, so it is refused everywhere.
#[test]
fn should_reject_a_drive_like_first_segment_when_on_any_platform() {
    for path in ["C:evil.dll", "c:/evil.dll", "C:"] {
        assert!(
            matches!(
                validate_relative_path(path),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "`{path}` must be rejected"
        );
    }
}

/// On Windows a `:` inside a name is a drive, a prefix or an alternate data stream.
#[cfg(windows)]
#[test]
fn should_reject_colon_in_a_segment_when_on_windows() {
    for path in ["pack.toml:secret", "mods/a.jar:stream", "mods/C:evil.dll"] {
        assert!(
            matches!(
                validate_relative_path(path),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "`{path}` must be rejected"
        );
    }
}

/// Elsewhere a `:` is part of a name a user's file may have.
#[cfg(not(windows))]
#[test]
fn should_accept_colon_in_a_file_name_when_not_on_windows() {
    for path in ["mods/a:b.jar", "pack.toml:secret", "mods/C:evil.dll"] {
        assert_eq!(
            validate_relative_path(path).unwrap(),
            path.split('/').collect::<PathBuf>(),
            "`{path}` must be accepted"
        );
    }
}

#[test]
fn should_reject_a_path_when_a_segment_ends_with_a_dot_or_space() {
    // Windows drops them: `.metadata.` and `.metadata ` are `.metadata` there.
    for path in [
        ".metadata./instance.json",
        ".metadata /instance.json",
        "mods/sodium.jar.",
        "mods./sodium.jar",
    ] {
        assert!(
            matches!(
                validate_relative_path(path),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "`{path}` must be rejected"
        );
    }
}

#[test]
fn should_reject_a_path_when_it_contains_a_nul_byte() {
    assert!(matches!(
        validate_relative_path("pack.toml\0.jar"),
        Err(InstanceError::InvalidRelativePath { .. })
    ));
}

// ── resolve_instance_file_path ──

#[test]
fn should_resolve_a_path_when_it_does_not_exist_yet() {
    let (_temp_dir, instance_dir) = setup();

    let resolved = resolve(&instance_dir, "mods/example.jar").expect("should resolve");

    let canonical_root = dunce::canonicalize(&instance_dir).unwrap();
    assert!(resolved.starts_with(&canonical_root));
    assert_eq!(resolved, canonical_root.join("mods").join("example.jar"));
}

#[test]
fn should_reject_traversal_before_touching_the_filesystem_when_the_path_climbs_out() {
    let (temp_dir, instance_dir) = setup();
    let outside = temp_dir.path().join("outside.txt");
    std::fs::write(&outside, b"secret").unwrap();

    assert!(matches!(
        resolve(&instance_dir, "../outside.txt"),
        Err(InstanceError::InvalidRelativePath { .. })
    ));
    assert!(
        outside.exists(),
        "the refused call must not have touched anything"
    );
}

/// The 8.3 short name of `.metadata` reaches it on disk without spelling it, so neither the
/// content rule nor the plugin write rule sees it. Skipped where the volume makes no short names.
#[cfg(windows)]
#[test]
fn should_reject_a_path_when_it_uses_a_short_name_of_metadata() {
    let (_temp_dir, instance_dir) = setup();
    let metadata_dir = instance_dir.join(".metadata");
    std::fs::create_dir_all(&metadata_dir).unwrap();
    std::fs::write(metadata_dir.join("instance.json"), b"{}").unwrap();

    let Some(short_name) = short_name_of(&metadata_dir) else {
        eprintln!("skipped: no 8.3 short name for {}", metadata_dir.display());
        return;
    };

    for relative in [
        format!("{short_name}/instance.json"),
        format!("{short_name}/new.json"),
    ] {
        assert!(
            matches!(
                resolve(&instance_dir, &relative),
                Err(InstanceError::InvalidRelativePath { .. })
            ),
            "`{relative}` must be rejected"
        );
    }
}

/// The 8.3 short name of `path`'s last component, if the file system gave it one.
#[cfg(windows)]
fn short_name_of(path: &Path) -> Option<String> {
    use std::os::windows::process::CommandExt;

    // `%~sI` expands to the short path; a volume without 8.3 names gives the long one back.
    // A raw argument: cmd does not take the quotes the usual argument escaping would produce.
    let output = std::process::Command::new("cmd")
        .raw_arg(format!(
            "/C for %I in (\"{}\") do @echo %~sI",
            path.display()
        ))
        .output()
        .ok()?;
    let short_path = String::from_utf8(output.stdout).ok()?;
    let short_name = Path::new(short_path.trim())
        .file_name()?
        .to_str()?
        .to_owned();

    (short_name != ".metadata" && !short_name.is_empty()).then_some(short_name)
}

#[test]
fn should_fail_when_the_instance_directory_does_not_exist() {
    let (temp_dir, _instance_dir) = setup();
    let missing = temp_dir.path().join("instances").join("no-such-instance");

    assert!(
        resolve_instance_file_path(
            &InstanceId::try_from("no-such-instance").unwrap(),
            &missing,
            "pack.toml"
        )
        .is_err(),
        "a missing instance directory must not be created on the fly"
    );
}

#[test]
fn should_reject_a_path_when_a_directory_symlink_points_out_of_the_instance() {
    let (temp_dir, instance_dir) = setup();

    let outside = temp_dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();

    let link = instance_dir.join("mods");
    link_dir(&outside, &link);

    // Passes the string rules — only canonicalisation can catch it.
    assert!(validate_relative_path("mods/escaped.txt").is_ok());
    assert!(
        matches!(
            resolve(&instance_dir, "mods/escaped.txt"),
            Err(InstanceError::PathEscapesInstance { .. })
        ),
        "a path through a symlinked directory must be refused"
    );
}

#[test]
fn should_reject_a_path_when_a_file_symlink_points_out_of_the_instance() {
    let (temp_dir, instance_dir) = setup();

    let outside = temp_dir.path().join("secret.txt");
    std::fs::write(&outside, b"secret").unwrap();

    let link = instance_dir.join("pack.toml");
    link_file(&outside, &link);

    assert!(matches!(
        resolve(&instance_dir, "pack.toml"),
        Err(InstanceError::PathEscapesInstance { .. })
    ));
}

#[test]
fn should_accept_a_path_when_a_symlink_stays_inside_the_instance() {
    let (_temp_dir, instance_dir) = setup();

    let real = instance_dir.join("real");
    std::fs::create_dir_all(&real).unwrap();

    let link = instance_dir.join("mods");
    link_dir(&real, &link);

    assert!(
        resolve(&instance_dir, "mods/example.jar").is_ok(),
        "a symlink that stays inside the instance is not an escape"
    );
}

// ── check_plugin_write_allowed ──

const PLUGIN_ID: &str = "packwiz";

fn check(relative_path: &str) -> Result<(), InstanceError> {
    check_plugin_write_allowed(PLUGIN_ID, relative_path)
}

#[test]
fn should_allow_a_plugin_write_when_the_path_is_outside_metadata() {
    for path in ["mods/example.jar", "pack.toml", "config/a/b.txt"] {
        assert!(check(path).is_ok(), "{path} should be allowed");
    }
}

#[test]
fn should_allow_a_plugin_write_when_the_path_is_in_its_own_metadata_subtree() {
    for path in [
        ".metadata/plugins/packwiz/packwiz.toml",
        ".metadata/plugins/packwiz/state/cache.bin",
    ] {
        assert!(check(path).is_ok(), "{path} should be allowed");
    }
}

#[test]
fn should_refuse_a_plugin_write_when_the_path_is_in_instance_metadata() {
    let err = check(".metadata/instance.json")
        .expect_err("instance.json belongs to the launcher, not to a plugin");
    assert!(
        matches!(err, InstanceError::InvalidRelativePath { .. }),
        "{err}"
    );
}

#[test]
fn should_refuse_a_plugin_write_when_the_path_is_in_another_plugins_directory() {
    for path in [
        ".metadata/plugins/curseforge/settings.toml",
        // Same directory on a case-insensitive filesystem — must not pass on the strength of
        // spelling alone.
        ".metadata/plugins/Packwiz/packwiz.toml",
    ] {
        assert!(
            check(path).is_err(),
            "{path} is not this plugin's own directory"
        );
    }
}

#[test]
fn should_refuse_a_plugin_write_when_the_path_is_the_plugins_directory_itself() {
    // Allowing these would mean writing a *file* called `packwiz`, `plugins` or `.metadata`
    // where the core expects a directory.
    for path in [
        ".metadata",
        ".metadata/plugins",
        ".metadata/plugins/packwiz",
    ] {
        assert!(
            check(path).is_err(),
            "{path} is a directory, not a file in it"
        );
    }
}

#[test]
fn should_recognise_metadata_when_its_case_differs() {
    // `.METADATA` is the same directory on Windows. It is refused rather than allowed under a
    // second spelling, even though it matches nothing the core writes.
    for path in [
        ".METADATA/instance.json",
        ".Metadata/plugins/packwiz/packwiz.toml",
    ] {
        assert!(check(path).is_err(), "{path} must not slip past on case");
    }
}

#[test]
fn should_refuse_a_path_when_a_traversal_leads_into_metadata() {
    // The segment rules run first, so this never reaches the `.metadata` comparison.
    assert!(matches!(
        check("mods/../.metadata/instance.json"),
        Err(InstanceError::InvalidRelativePath { .. })
    ));
}

#[test]
fn should_treat_a_backslash_as_a_separator_when_checking_plugin_writes() {
    assert!(
        check(r".metadata\instance.json").is_err(),
        "a Windows-style separator must not hide the `.metadata` segment"
    );
    assert!(
        check(r".metadata\plugins\packwiz\packwiz.toml").is_ok(),
        "the plugin's own subtree is its own whichever separator is used"
    );
}
