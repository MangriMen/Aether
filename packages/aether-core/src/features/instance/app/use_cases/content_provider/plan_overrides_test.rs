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

fn archive_at(dir: &Path, bytes: Vec<u8>, subdir: &str) -> OverrideLocation {
    let archive_path = dir.join("pack.mrpack");
    std::fs::write(&archive_path, bytes).unwrap();

    OverrideLocation::Archive {
        archive_path,
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
    let directory = OverrideLocation::Directory { path: directory };

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
        matches!(&err, InstanceError::Storage(reason) if reason.contains("larger than its declared")),
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
    let (overrides, _) = listed_directory(&dir);

    let staged = stage_sources(
        &dir,
        vec![OverrideLocation::Directory { path: overrides }],
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
        path: overrides.clone(),
    };
    let files = list_overrides(&source, 10).await.unwrap();

    std::fs::remove_dir_all(overrides.join("sub")).unwrap();
    link_dir(&outside, &overrides.join("sub"));

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
        path: overrides.clone(),
    };
    let files = list_overrides(&source, 10).await.unwrap();

    std::fs::remove_dir_all(&overrides).unwrap();
    link_dir(&outside, &overrides);

    let err = stage_overrides(vec![(source, files)], &dir.path().join("staging"), 100)
        .await
        .err()
        .unwrap();

    assert!(matches!(err, InstanceError::Storage(_)), "{err:?}");
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
    let directory = OverrideLocation::Directory { path: directory };

    for source in [archive, directory] {
        assert_eq!(list_overrides(&source, 2).await.unwrap().len(), 3);
        assert_eq!(list_overrides(&source, 5).await.unwrap().len(), 5);
    }
}
