//! The entry count a zip archive declares, read without its central directory.
//!
//! `ZipArchive::new` reads the whole central directory and keeps a record per entry, so an
//! archive of millions of tiny entries costs gigabytes before any limit on files can apply.
//! Asking first lets the caller refuse such an archive cheaply. The count comes from the zip
//! crate's own end-record search (`ZipIterable`), so it is the very record `ZipArchive` would
//! go by, whatever the comment, ZIP64 fields or junk before the archive look like.

use std::io::{Read, Seek};

use zip::unstable::read::ZipIterable;

/// The number of entries the archive in `reader` declares, or `None` when it cannot be told
/// (the archive reader will say what is wrong with it). The reader's position is not kept.
pub fn declared_entries<R: Read + Seek>(reader: &mut R) -> Option<usize> {
    ZipIterable::try_new(reader, zip::read::Config::default())
        .ok()
        .map(|archive| archive.len())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::*;

    fn archive_of(entries: usize, comment: &str) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for index in 0..entries {
            writer.start_file(format!("f{index}"), options).unwrap();
            writer.write_all(b"x").unwrap();
        }
        writer.set_raw_comment(comment.as_bytes().into()).unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn should_report_the_entries_when_the_archive_is_small() {
        let mut reader = Cursor::new(archive_of(3, ""));

        assert_eq!(declared_entries(&mut reader), Some(3));
    }

    #[test]
    fn should_report_the_entries_when_the_archive_has_a_comment() {
        let mut reader = Cursor::new(archive_of(2, "a comment after the end record"));

        assert_eq!(declared_entries(&mut reader), Some(2));
    }

    #[test]
    fn should_report_the_entries_when_the_archive_is_empty() {
        let mut reader = Cursor::new(archive_of(0, ""));

        assert_eq!(declared_entries(&mut reader), Some(0));
    }

    #[test]
    fn should_report_the_entries_when_the_count_needs_zip64() {
        let mut reader = Cursor::new(archive_of(70_000, ""));

        assert_eq!(declared_entries(&mut reader), Some(70_000));
    }

    #[test]
    fn should_report_the_entries_when_the_comment_is_as_long_as_it_can_be() {
        let comment = "c".repeat(usize::from(u16::MAX));
        let mut reader = Cursor::new(archive_of(70_000, &comment));

        assert_eq!(declared_entries(&mut reader), Some(70_000));
    }

    #[test]
    fn should_report_nothing_when_there_is_no_end_record() {
        let mut reader = Cursor::new(b"not an archive at all".to_vec());

        assert_eq!(declared_entries(&mut reader), None);
    }
}
