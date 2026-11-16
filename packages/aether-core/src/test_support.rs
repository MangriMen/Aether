//! Helpers that the tests of more than one slice share.

use std::path::Path;

/// Makes `link` a directory link to `target`.
///
/// A symlink, or where Windows refuses one without Developer Mode or elevation, a junction, which
/// needs neither and is resolved the same way when a path is canonicalised. Panics when no link
/// can be made: a test that went on without one would pass without proving anything.
pub(crate) fn link_dir(target: &Path, link: &Path) {
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_dir(target, link).is_ok() {
            return;
        }
        let junction = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(link)
            .arg(target)
            .output()
            .is_ok_and(|output| output.status.success());
        assert!(
            junction,
            "could not link {} to {}",
            link.display(),
            target.display()
        );
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).unwrap_or_else(|err| {
            panic!(
                "could not link {} to {}: {err}",
                link.display(),
                target.display()
            )
        });
    }
}

/// Makes `link` a symlink to the file `target`.
///
/// Windows allows a file symlink only with Developer Mode or elevation, and a junction cannot
/// stand in for one, so there the test fails with the reason rather than passing unchecked.
pub(crate) fn link_file(target: &Path, link: &Path) {
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_file(target, link);
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(target, link);

    linked.unwrap_or_else(|err| {
        panic!(
            "could not link {} to {}: {err}. On Windows, turn on Developer Mode or run elevated.",
            link.display(),
            target.display()
        )
    });
}

/// Makes renaming the file `path` fail, as a jar held open by a running game does on Windows,
/// until the guard is dropped. `None` where this cannot be done: on Unix a read-only directory
/// stops nobody running as root, and a test that went on would not prove anything.
#[must_use = "the rename works again once the guard is dropped"]
// Windows can always hold the file; only Unix may have to give up.
#[cfg_attr(windows, expect(clippy::unnecessary_wraps))]
pub(crate) fn block_rename(path: &Path) -> Option<RenameBlock> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        // Without `FILE_SHARE_DELETE` (std shares it by default) a file cannot be renamed.
        const FILE_SHARE_READ: u32 = 1;
        Some(RenameBlock(
            std::fs::OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .open(path)
                .expect("could not hold the file open"),
        ))
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let dir = path.parent().expect("a file has a directory").to_path_buf();
        let original = std::fs::metadata(&dir).unwrap().permissions();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        // Root is not stopped by it.
        let probe = dir.join(".rename-block-probe");
        if std::fs::write(&probe, b"").is_ok() {
            std::fs::remove_file(&probe).unwrap();
            std::fs::set_permissions(&dir, original).unwrap();
            eprintln!("skipped: a read-only directory does not stop root");
            return None;
        }

        Some(RenameBlock { dir, original })
    }
}

#[cfg(windows)]
pub(crate) struct RenameBlock(#[allow(dead_code)] std::fs::File);

#[cfg(unix)]
pub(crate) struct RenameBlock {
    dir: std::path::PathBuf,
    original: std::fs::Permissions,
}

#[cfg(unix)]
impl Drop for RenameBlock {
    fn drop(&mut self) {
        let _ = std::fs::set_permissions(&self.dir, self.original.clone());
    }
}
