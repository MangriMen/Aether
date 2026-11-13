use std::io::{Cursor, Write};

use tempfile::TempDir;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use super::*;
use crate::test_support::link_dir;

/// A stored (uncompressed) zip with `entries` of `size` bytes each.
fn zip_bytes(entries: &[&str], size: usize) -> Vec<u8> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for name in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(&vec![b'x'; size]).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// Rewrite the uncompressed size every header of a stored `zip` declares, as a bomb does:
/// the data, its compressed size and its checksum stay as they are, so it still reads whole.
fn declare_size(mut zip: Vec<u8>, declared: u32) -> Vec<u8> {
    // Local file header and central directory header, and where each keeps the size.
    for (signature, offset) in [(b"PK\x03\x04", 22), (b"PK\x01\x02", 24)] {
        let starts: Vec<usize> = zip
            .windows(4)
            .enumerate()
            .filter(|(_, window)| window == signature)
            .map(|(start, _)| start)
            .collect();
        for start in starts {
            zip[start + offset..start + offset + 4].copy_from_slice(&declared.to_le_bytes());
        }
    }
    zip
}

/// `relative` (`/`-separated) under `root`.
fn anchored(root: &Path, relative: &str) -> AnchoredPath {
    AnchoredPath::new(root.to_owned(), relative.split('/').collect()).unwrap()
}

fn archive_at(dir: &Path, bytes: Vec<u8>, subdir: &str) -> OverrideLocation {
    std::fs::write(dir.join("pack.mrpack"), bytes).unwrap();

    OverrideLocation::Archive {
        archive_path: anchored(dir, "pack.mrpack"),
        subdir: subdir.into(),
    }
}

/// `overrides/a.txt` and `overrides/b.txt`, 100 bytes each.
fn archive(dir: &Path) -> OverrideLocation {
    archive_at(
        dir,
        zip_bytes(&["overrides/a.txt", "overrides/b.txt"], 100),
        "overrides",
    )
}

async fn stage_sources(
    dir: &TempDir,
    sources: Vec<OverrideLocation>,
    max_bytes: u64,
) -> Result<Vec<StagedOverride>, InstanceError> {
    let mut listed = Vec::new();
    for source in sources {
        let files = list_overrides(&source, usize::MAX).await.unwrap();
        listed.push((source, files));
    }

    stage_overrides(listed, &dir.path().join("staging"), max_bytes).await
}

async fn stage(max_bytes: u64) -> Result<Vec<StagedOverride>, InstanceError> {
    let dir = TempDir::new().unwrap();
    let source = archive(dir.path());

    stage_sources(&dir, vec![source], max_bytes).await
}

#[tokio::test]
async fn should_stage_overrides_when_they_are_within_the_limit() {
    assert_eq!(stage(200).await.unwrap().len(), 2);
}

/// Counted over every file of the archive, not per file: each fits, both do not.
#[tokio::test]
async fn should_refuse_overrides_when_an_archive_unpacks_past_the_limit() {
    let err = stage(150).await.err().unwrap();

    assert!(
        matches!(
            &err,
            InstanceError::FileTooLarge { path, limit: 150, .. } if path == "b.txt"
        ),
        "{err:?}"
    );
}

/// Counted over the sources of a plan together: an archive's subdirectory and a directory
/// each fit, both do not.
#[tokio::test]
async fn should_refuse_overrides_when_all_sources_together_pass_the_limit() {
    let dir = TempDir::new().unwrap();
    let archive = archive(dir.path());
    let directory = dir.path().join("plugin-overrides");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("c.txt"), [b'x'; 100]).unwrap();
    let directory = OverrideLocation::Directory {
        path: anchored(dir.path(), "plugin-overrides"),
    };

    let staged = stage_sources(&dir, vec![archive.clone(), directory.clone()], 300).await;
    assert_eq!(staged.unwrap().len(), 3);

    let err = stage_sources(&dir, vec![archive, directory], 250)
        .await
        .err()
        .unwrap();
    assert!(
        matches!(
            &err,
            InstanceError::FileTooLarge { path, limit: 250, .. } if path == "c.txt"
        ),
        "{err:?}"
    );
}

/// The headers are the author's: an archive declaring 10 bytes an entry, within the limit,
/// is refused by the 100 an entry it unpacks to (zip 9 itself stops a read past the declared size).
#[tokio::test]
async fn should_refuse_an_archive_when_it_unpacks_to_more_than_it_declares() {
    let dir = TempDir::new().unwrap();
    let bytes = declare_size(zip_bytes(&["overrides/a.txt", "overrides/b.txt"], 100), 10);
    let source = archive_at(dir.path(), bytes, "overrides");

    let files = list_overrides(&source, 10).await.unwrap();
    assert_eq!(
        files
            .iter()
            .map(|file| file.declared_size)
            .collect::<Vec<_>>(),
        [10, 10],
        "the archive understates its entries"
    );

    let err = stage_overrides(vec![(source, files)], &dir.path().join("staging"), 150)
        .await
        .err()
        .unwrap();
    assert!(
        matches!(&err, InstanceError::Storage(StorageError::Io(_))),
        "{err:?}"
    );
}

/// `sub/a.txt` of a plugin's overrides directory, listed, and a secret next to it outside.
fn listed_directory(dir: &TempDir) -> (PathBuf, PathBuf) {
    let overrides = dir.path().join("plugin-overrides");
    std::fs::create_dir_all(overrides.join("sub")).unwrap();
    std::fs::write(overrides.join("sub").join("a.txt"), "override").unwrap();

    let outside = dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("a.txt"), "secret").unwrap();

    (overrides, outside)
}

#[tokio::test]
async fn should_stage_the_files_of_a_directory_override() {
    let dir = TempDir::new().unwrap();
    listed_directory(&dir);

    let staged = stage_sources(
        &dir,
        vec![OverrideLocation::Directory {
            path: anchored(dir.path(), "plugin-overrides"),
        }],
        100,
    )
    .await
    .unwrap();

    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].relative_path, "sub/a.txt");
    assert_eq!(
        std::fs::read_to_string(&staged[0].path).unwrap(),
        "override"
    );
}

/// A directory on the path, swapped for a link out of the source after it was listed, is
/// not followed: the file it leads to is not read.
#[tokio::test]
async fn should_refuse_an_override_when_a_directory_on_its_path_became_a_link() {
    let dir = TempDir::new().unwrap();
    let (overrides, outside) = listed_directory(&dir);
    let source = OverrideLocation::Directory {
        path: anchored(dir.path(), "plugin-overrides"),
    };
    let files = list_overrides(&source, 10).await.unwrap();

    std::fs::remove_dir_all(overrides.join("sub")).unwrap();
    link_dir(&outside, &overrides.join("sub"));
    assert!(
        std::fs::symlink_metadata(overrides.join("sub"))
            .unwrap()
            .file_type()
            .is_symlink()
    );

    let staging = dir.path().join("staging");
    let err = stage_overrides(vec![(source, files)], &staging, 100)
        .await
        .err()
        .unwrap();

    assert!(matches!(err, InstanceError::Storage(_)), "{err:?}");
    let staged_secret = std::fs::read_dir(&staging)
        .unwrap()
        .any(|entry| std::fs::read_to_string(entry.unwrap().path()).unwrap() == "secret");
    assert!(!staged_secret);
}

/// The root of the source, swapped for a link after it was listed, is not followed either.
#[tokio::test]
async fn should_refuse_overrides_when_their_directory_became_a_link() {
    let dir = TempDir::new().unwrap();
    let (overrides, outside) = listed_directory(&dir);
    std::fs::create_dir_all(outside.join("sub")).unwrap();
    std::fs::write(outside.join("sub").join("a.txt"), "secret").unwrap();
    let source = OverrideLocation::Directory {
        path: anchored(dir.path(), "plugin-overrides"),
    };
    let files = list_overrides(&source, 10).await.unwrap();

    std::fs::remove_dir_all(&overrides).unwrap();
    link_dir(&outside, &overrides);
    assert!(
        std::fs::symlink_metadata(&overrides)
            .unwrap()
            .file_type()
            .is_symlink()
    );

    let err = stage_overrides(vec![(source, files)], &dir.path().join("staging"), 100)
        .await
        .err()
        .unwrap();

    assert!(matches!(err, InstanceError::Storage(_)), "{err:?}");
}

/// A plugin's source, anchored at the root of its cache: `packs/pack.mrpack` and
/// `packs/overrides/sub/a.txt`, and a `secret` directory outside the root holding the same
/// names with other content.
fn plugin_cache(dir: &TempDir) -> (PathBuf, PathBuf) {
    let root = dir.path().join("cache");
    let packs = root.join("packs");
    std::fs::create_dir_all(packs.join("overrides").join("sub")).unwrap();
    std::fs::write(
        packs.join("pack.mrpack"),
        zip_bytes(&["overrides/a.txt"], 10),
    )
    .unwrap();
    std::fs::write(
        packs.join("overrides").join("sub").join("a.txt"),
        "override",
    )
    .unwrap();

    let secret = dir.path().join("secret");
    std::fs::create_dir_all(secret.join("overrides").join("sub")).unwrap();
    std::fs::write(
        secret.join("pack.mrpack"),
        zip_bytes(&["overrides/a.txt", "overrides/secret.txt"], 10),
    )
    .unwrap();
    std::fs::write(secret.join("overrides").join("sub").join("a.txt"), "secret").unwrap();

    (root, secret)
}

/// A directory between the root and the source, swapped for a link out of the root after the
/// plugin's path was checked, is not followed: neither source behind it is read.
#[tokio::test]
async fn should_refuse_overrides_when_a_parent_of_their_source_became_a_link() {
    let dir = TempDir::new().unwrap();
    let (root, secret) = plugin_cache(&dir);
    let archive = OverrideLocation::Archive {
        archive_path: anchored(&root, "packs/pack.mrpack"),
        subdir: "overrides".into(),
    };
    let directory = OverrideLocation::Directory {
        path: anchored(&root, "packs/overrides"),
    };
    let listed = vec![
        (archive.clone(), list_overrides(&archive, 10).await.unwrap()),
        (
            directory.clone(),
            list_overrides(&directory, 10).await.unwrap(),
        ),
    ];

    std::fs::remove_dir_all(root.join("packs")).unwrap();
    link_dir(&secret, &root.join("packs"));
    assert!(
        std::fs::symlink_metadata(root.join("packs"))
            .unwrap()
            .file_type()
            .is_symlink()
    );

    for source in [&archive, &directory] {
        let err = list_overrides(source, 10).await.err().unwrap();
        assert!(matches!(err, InstanceError::Storage(_)), "{err:?}");
    }
    for (source, files) in listed {
        let err = stage_overrides(vec![(source, files)], &dir.path().join("staging"), 100)
            .await
            .err()
            .unwrap();
        assert!(matches!(err, InstanceError::Storage(_)), "{err:?}");
    }
}

/// The archive, swapped for a link after it was listed, is not followed.
#[tokio::test]
async fn should_refuse_an_archive_when_it_became_a_link() {
    let dir = TempDir::new().unwrap();
    let source = archive(dir.path());
    let files = list_overrides(&source, 10).await.unwrap();

    let archive_path = dir.path().join("pack.mrpack");
    let outside = dir.path().join("outside.zip");
    std::fs::rename(&archive_path, &outside).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &archive_path).unwrap();
    #[cfg(windows)]
    {
        // A file symlink needs the privilege, or Developer Mode, on Windows.
        const ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;
        if let Err(err) = std::os::windows::fs::symlink_file(&outside, &archive_path) {
            if err.raw_os_error() == Some(ERROR_PRIVILEGE_NOT_HELD) {
                eprintln!("skipped: creating a file symlink is not allowed here");
                return;
            }
            panic!("could not link {}: {err}", archive_path.display());
        }
    }
    assert!(
        std::fs::symlink_metadata(&archive_path)
            .unwrap()
            .file_type()
            .is_symlink()
    );

    let err = stage_overrides(vec![(source, files)], &dir.path().join("staging"), 1000)
        .await
        .err()
        .unwrap();

    assert!(
        matches!(&err, InstanceError::Storage(StorageError::Io(_))),
        "{err:?}"
    );
}

/// A directory in the archive's place is refused when it is opened, before it is read as a zip,
/// whether it was there when listed or put there after.
#[tokio::test]
async fn should_refuse_an_archive_when_it_is_a_directory() {
    let dir = TempDir::new().unwrap();
    let source = archive(dir.path());
    let files = list_overrides(&source, 10).await.unwrap();

    let archive_path = dir.path().join("pack.mrpack");
    std::fs::remove_file(&archive_path).unwrap();
    std::fs::create_dir_all(archive_path.join("overrides")).unwrap();
    std::fs::write(archive_path.join("overrides").join("a.txt"), "x").unwrap();
    assert!(std::fs::symlink_metadata(&archive_path).unwrap().is_dir());

    let is_refused_open =
        |err: &InstanceError| matches!(err, InstanceError::Storage(StorageError::Io(_)));
    let err = list_overrides(&source, 10).await.err().unwrap();
    assert!(is_refused_open(&err), "{err:?}");
    let err = stage_overrides(vec![(source, files)], &dir.path().join("staging"), 1000)
        .await
        .err()
        .unwrap();
    assert!(is_refused_open(&err), "{err:?}");
}

/// Listing stops one file past the limit, in an archive and in a directory alike.
#[tokio::test]
async fn should_list_no_more_than_one_file_past_the_limit_when_the_limit_is_exceeded() {
    let dir = TempDir::new().unwrap();
    let names: Vec<String> = (0..5)
        .map(|index| format!("overrides/{index}.txt"))
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let archive = archive_at(dir.path(), zip_bytes(&names, 1), "overrides");

    let directory = dir.path().join("plugin-overrides");
    std::fs::create_dir_all(directory.join("sub")).unwrap();
    for index in 0..5 {
        std::fs::write(directory.join("sub").join(format!("{index}.txt")), "x").unwrap();
    }
    let directory = OverrideLocation::Directory {
        path: anchored(dir.path(), "plugin-overrides"),
    };

    for source in [archive, directory] {
        assert_eq!(list_overrides(&source, 2).await.unwrap().len(), 3);
        assert_eq!(list_overrides(&source, 5).await.unwrap().len(), 5);
    }
}
