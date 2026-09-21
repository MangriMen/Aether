//! Path checks behind `write_instance_file` / `read_instance_file` (T-1.3).

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::features::instance::domain::InstanceError;
use crate::features::instance::infra::{resolve_instance_file_path, validate_relative_path};

const INSTANCE_ID: &str = "inst-1";

fn setup() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let instance_dir = temp_dir.path().join("instances").join(INSTANCE_ID);
    std::fs::create_dir_all(&instance_dir).expect("Failed to create instance dir");
    (temp_dir, instance_dir)
}

fn resolve(instance_dir: &Path, relative_path: &str) -> Result<PathBuf, InstanceError> {
    resolve_instance_file_path(INSTANCE_ID, instance_dir, relative_path)
}

/// Create a directory symlink, or report that this host will not let us.
///
/// On Windows this needs Developer Mode or elevation, so the symlink tests report themselves as
/// skipped rather than failing on a machine that simply is not allowed to make one.
fn try_symlink_dir(target: &Path, link: &Path) -> bool {
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(target, link).is_ok()
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
}

fn try_symlink_file(target: &Path, link: &Path) -> bool {
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
}

// ── validate_relative_path ──

#[test]
fn should_accept_a_plain_relative_path() {
    let path = validate_relative_path("mods/example.jar").expect("plain path should be accepted");
    assert_eq!(path, PathBuf::from("mods").join("example.jar"));
}

#[test]
fn should_accept_a_single_file_name() {
    assert_eq!(
        validate_relative_path("pack.toml").expect("file name should be accepted"),
        PathBuf::from("pack.toml")
    );
}

#[test]
fn should_accept_backslashes_as_separators() {
    assert_eq!(
        validate_relative_path(r"config\mod\settings.json").expect("should be accepted"),
        PathBuf::from("config").join("mod").join("settings.json")
    );
}

#[test]
fn should_reject_parent_directory_segments() {
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
fn should_reject_current_directory_segments() {
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
fn should_reject_absolute_paths() {
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
fn should_reject_wasi_style_paths() {
    // The trap from the task card: plugins speak `/instances/…` and `/mnt/d/…` to WASI, and
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
fn should_reject_empty_and_separator_only_paths() {
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

#[test]
fn should_reject_colon_in_a_segment() {
    // Drive-relative paths and Windows alternate data streams both hide behind a `:`.
    for path in ["C:evil.dll", "pack.toml:secret", "mods/a.jar:stream"] {
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
fn should_reject_nul_byte() {
    assert!(matches!(
        validate_relative_path("pack.toml\0.jar"),
        Err(InstanceError::InvalidRelativePath { .. })
    ));
}

// ── resolve_instance_file_path ──

#[test]
fn should_resolve_a_path_that_does_not_exist_yet() {
    let (_temp_dir, instance_dir) = setup();

    let resolved = resolve(&instance_dir, "mods/example.jar").expect("should resolve");

    let canonical_root = dunce::canonicalize(&instance_dir).unwrap();
    assert!(resolved.starts_with(&canonical_root));
    assert_eq!(resolved, canonical_root.join("mods").join("example.jar"));
}

#[test]
fn should_reject_traversal_before_touching_the_filesystem() {
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

#[test]
fn should_fail_when_the_instance_directory_does_not_exist() {
    let (temp_dir, _instance_dir) = setup();
    let missing = temp_dir.path().join("instances").join("no-such-instance");

    assert!(
        resolve_instance_file_path("no-such-instance", &missing, "pack.toml").is_err(),
        "a missing instance directory must not be created on the fly"
    );
}

#[test]
fn should_reject_a_directory_symlink_pointing_out_of_the_instance() {
    let (temp_dir, instance_dir) = setup();

    let outside = temp_dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();

    let link = instance_dir.join("mods");
    if !try_symlink_dir(&outside, &link) {
        eprintln!("skipped: this host does not allow creating directory symlinks");
        return;
    }

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
fn should_reject_a_file_symlink_pointing_out_of_the_instance() {
    let (temp_dir, instance_dir) = setup();

    let outside = temp_dir.path().join("secret.txt");
    std::fs::write(&outside, b"secret").unwrap();

    let link = instance_dir.join("pack.toml");
    if !try_symlink_file(&outside, &link) {
        eprintln!("skipped: this host does not allow creating file symlinks");
        return;
    }

    assert!(matches!(
        resolve(&instance_dir, "pack.toml"),
        Err(InstanceError::PathEscapesInstance { .. })
    ));
}

#[test]
fn should_accept_a_symlink_that_stays_inside_the_instance() {
    let (_temp_dir, instance_dir) = setup();

    let real = instance_dir.join("real");
    std::fs::create_dir_all(&real).unwrap();

    let link = instance_dir.join("mods");
    if !try_symlink_dir(&real, &link) {
        eprintln!("skipped: this host does not allow creating directory symlinks");
        return;
    }

    assert!(
        resolve(&instance_dir, "mods/example.jar").is_ok(),
        "a symlink that stays inside the instance is not an escape"
    );
}
