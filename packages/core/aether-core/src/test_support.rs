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
