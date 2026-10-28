//! Path checks behind `write_instance_file` / `read_instance_file`.

use std::path::{Path, PathBuf};

use rstest::rstest;
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

#[rstest]
#[case::plain_relative_path("mods/example.jar", PathBuf::from("mods").join("example.jar"))]
#[case::single_file_name("pack.toml", PathBuf::from("pack.toml"))]
#[case::backslash_separators(
    r"config\mod\settings.json",
    PathBuf::from("config").join("mod").join("settings.json")
)]
fn should_accept_a_path_when_it_is_valid(#[case] path: &str, #[case] expected: PathBuf) {
    assert_eq!(
        validate_relative_path(path).expect("path should be accepted"),
        expected
    );
}

#[rstest]
// parent directory segments
#[case::parent_at_start("../evil.txt")]
#[case::parent_climbs_out("mods/../../evil.txt")]
#[case::parent_in_the_middle("mods/../evil.txt")]
#[case::parent_with_backslashes(r"mods\..\..\evil.txt")]
#[case::parent_alone("..")]
// current directory segments
#[case::current_at_start("./pack.toml")]
#[case::current_in_the_middle("mods/./example.jar")]
#[case::current_alone(".")]
// absolute paths
#[case::absolute_unix("/etc/passwd")]
#[case::absolute_instances("/instances/inst-1/pack.toml")]
#[case::absolute_backslash(r"\Windows\System32\evil.dll")]
#[case::absolute_drive_slash("C:/Windows/System32/evil.dll")]
#[case::absolute_drive_backslash(r"C:\Windows\System32\evil.dll")]
#[case::absolute_unc(r"\\server\share\evil.dll")]
#[case::absolute_verbatim(r"\\?\C:\evil.dll")]
// WASI style: plugins speak `/instances/…` and `/mnt/d/…` to WASI, and neither form may be
// accepted here
#[case::wasi_instances("/instances/inst-1/mods/a.jar")]
#[case::wasi_mnt("/mnt/d/instances/a.jar")]
// empty or only separators
#[case::empty("")]
#[case::slash("/")]
#[case::backslash(r"\")]
#[case::double_slash("mods//a.jar")]
#[case::trailing_slash("mods/")]
#[case::double_backslash(r"mods\\a.jar")]
// a first segment spelled as a drive is absolute on Windows, so it is refused everywhere
#[case::drive_like_without_separator("C:evil.dll")]
#[case::drive_like_lowercase("c:/evil.dll")]
#[case::drive_like_alone("C:")]
// Windows drops trailing dots and spaces: `.metadata.` and `.metadata ` are `.metadata` there
#[case::metadata_trailing_dot(".metadata./instance.json")]
#[case::metadata_trailing_space(".metadata /instance.json")]
#[case::file_trailing_dot("mods/sodium.jar.")]
#[case::folder_trailing_dot("mods./sodium.jar")]
#[case::nul_byte("pack.toml\0.jar")]
fn should_reject_a_path_when_it_is_invalid(#[case] path: &str) {
    assert!(
        matches!(
            validate_relative_path(path),
            Err(InstanceError::InvalidRelativePath { .. })
        ),
        "`{path}` must be rejected"
    );
}

/// On Windows a `:` inside a name is a drive, a prefix or an alternate data stream.
#[cfg(windows)]
#[rstest]
#[case::alternate_data_stream("pack.toml:secret")]
#[case::stream_in_a_folder("mods/a.jar:stream")]
#[case::drive_in_a_segment("mods/C:evil.dll")]
fn should_reject_colon_in_a_segment_when_on_windows(#[case] path: &str) {
    assert!(
        matches!(
            validate_relative_path(path),
            Err(InstanceError::InvalidRelativePath { .. })
        ),
        "`{path}` must be rejected"
    );
}

/// Elsewhere a `:` is part of a name a user's file may have.
#[cfg(not(windows))]
#[rstest]
#[case::colon_in_a_file_name("mods/a:b.jar")]
#[case::stream_like_name("pack.toml:secret")]
#[case::drive_like_in_a_segment("mods/C:evil.dll")]
fn should_accept_colon_in_a_file_name_when_not_on_windows(#[case] path: &str) {
    assert_eq!(
        validate_relative_path(path).unwrap(),
        path.split('/').collect::<PathBuf>(),
        "`{path}` must be accepted"
    );
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

#[rstest]
#[case::outside_metadata_mod("mods/example.jar")]
#[case::outside_metadata_pack("pack.toml")]
#[case::outside_metadata_nested("config/a/b.txt")]
#[case::own_metadata_file(".metadata/plugins/packwiz/packwiz.toml")]
#[case::own_metadata_nested(".metadata/plugins/packwiz/state/cache.bin")]
// the plugin's own subtree is its own whichever separator is used
#[case::own_metadata_backslashes(r".metadata\plugins\packwiz\packwiz.toml")]
fn should_allow_a_plugin_write_when_the_path_is_permitted(#[case] path: &str) {
    assert!(check(path).is_ok(), "{path} should be allowed");
}

#[rstest]
#[case::another_plugins_directory(
    ".metadata/plugins/curseforge/settings.toml",
    "is not this plugin's own directory"
)]
// Same directory on a case-insensitive filesystem — must not pass on the strength of spelling
// alone.
#[case::another_plugins_directory_by_case(
    ".metadata/plugins/Packwiz/packwiz.toml",
    "is not this plugin's own directory"
)]
// Allowing these would mean writing a *file* called `packwiz`, `plugins` or `.metadata`
// where the core expects a directory.
#[case::metadata_directory(".metadata", "is a directory, not a file in it")]
#[case::plugins_directory(".metadata/plugins", "is a directory, not a file in it")]
#[case::own_plugin_directory(".metadata/plugins/packwiz", "is a directory, not a file in it")]
// `.METADATA` is the same directory on Windows. It is refused rather than allowed under a
// second spelling, even though it matches nothing the core writes.
#[case::metadata_upper_case(".METADATA/instance.json", "must not slip past on case")]
#[case::metadata_mixed_case(".Metadata/plugins/packwiz/packwiz.toml", "must not slip past on case")]
// a Windows-style separator must not hide the `.metadata` segment
#[case::metadata_backslash(r".metadata\instance.json", "must not hide the `.metadata` segment")]
fn should_refuse_a_plugin_write_when_the_path_is_not_permitted(
    #[case] path: &str,
    #[case] reason: &str,
) {
    assert!(check(path).is_err(), "{path}: {reason}");
}

#[rstest]
// instance.json belongs to the launcher, not to a plugin
#[case::instance_metadata(".metadata/instance.json")]
// The segment rules run first, so this never reaches the `.metadata` comparison.
#[case::traversal_into_metadata("mods/../.metadata/instance.json")]
fn should_refuse_a_plugin_write_with_an_invalid_path_error_when_the_path_is_forbidden(
    #[case] path: &str,
) {
    let err = check(path).expect_err("the path must be refused");
    assert!(
        matches!(err, InstanceError::InvalidRelativePath { .. }),
        "{err}"
    );
}
