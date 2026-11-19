//! The entry count a zip archive declares, read from its end record alone.
//!
//! `ZipArchive::new` reads the whole central directory and keeps a record per entry, so an
//! archive of millions of tiny entries costs gigabytes before any limit on files can apply.
//! Asking the end record first lets the caller refuse such an archive cheaply.

use std::io::{self, Read, Seek, SeekFrom};

const EOCD_SIGNATURE: [u8; 4] = *b"PK\x05\x06";
const EOCD_LEN: usize = 22;
const MAX_COMMENT_LEN: usize = u16::MAX as usize;
const ZIP64_LOCATOR_SIGNATURE: [u8; 4] = *b"PK\x06\x07";
const ZIP64_LOCATOR_LEN: usize = 20;
const ZIP64_EOCD_SIGNATURE: [u8; 4] = *b"PK\x06\x06";
const ZIP64_EOCD_LEN: usize = 56;

/// The number of entries the end record of the archive in `reader` declares, or `None` when no
/// record of a known shape is found (the archive reader will say what is wrong with it).
///
/// The reader's position is not kept.
pub fn declared_entries<R: Read + Seek>(reader: &mut R) -> io::Result<Option<u64>> {
    let len = reader.seek(SeekFrom::End(0))?;
    let tail_len = usize::try_from(len).map_or(EOCD_LEN + MAX_COMMENT_LEN, |len| {
        len.min(EOCD_LEN + MAX_COMMENT_LEN)
    });
    reader.seek(SeekFrom::Start(len - tail_len as u64))?;
    let mut tail = vec![0_u8; tail_len];
    reader.read_exact(&mut tail)?;

    let Some(eocd) = tail
        .windows(EOCD_LEN)
        .rposition(|window| window[..4] == EOCD_SIGNATURE)
    else {
        return Ok(None);
    };
    let total = u64::from(u16::from_le_bytes([tail[eocd + 10], tail[eocd + 11]]));
    if total != u64::from(u16::MAX) {
        return Ok(Some(total));
    }

    // 0xFFFF says the count does not fit: the ZIP64 record, which the locator before the end
    // record points to, holds it.
    let Some(locator) = eocd.checked_sub(ZIP64_LOCATOR_LEN) else {
        return Ok(Some(total));
    };
    if tail[locator..locator + 4] != ZIP64_LOCATOR_SIGNATURE {
        return Ok(Some(total));
    }
    let offset = u64::from_le_bytes(tail[locator + 8..locator + 16].try_into().expect("8 bytes"));
    if offset.saturating_add(ZIP64_EOCD_LEN as u64) > len {
        return Ok(None);
    }

    reader.seek(SeekFrom::Start(offset))?;
    let mut record = [0_u8; ZIP64_EOCD_LEN];
    reader.read_exact(&mut record)?;
    if record[..4] != ZIP64_EOCD_SIGNATURE {
        return Ok(None);
    }
    Ok(Some(u64::from_le_bytes(
        record[32..40].try_into().expect("8 bytes"),
    )))
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

        assert_eq!(declared_entries(&mut reader).unwrap(), Some(3));
    }

    #[test]
    fn should_report_the_entries_when_the_archive_has_a_comment() {
        let mut reader = Cursor::new(archive_of(2, "a comment after the end record"));

        assert_eq!(declared_entries(&mut reader).unwrap(), Some(2));
    }

    #[test]
    fn should_report_the_entries_when_the_archive_is_empty() {
        let mut reader = Cursor::new(archive_of(0, ""));

        assert_eq!(declared_entries(&mut reader).unwrap(), Some(0));
    }

    #[test]
    fn should_report_the_entries_when_the_count_needs_zip64() {
        let mut reader = Cursor::new(archive_of(70_000, ""));

        assert_eq!(declared_entries(&mut reader).unwrap(), Some(70_000));
    }

    #[test]
    fn should_report_nothing_when_there_is_no_end_record() {
        let mut reader = Cursor::new(b"not an archive at all".to_vec());

        assert_eq!(declared_entries(&mut reader).unwrap(), None);
    }
}
