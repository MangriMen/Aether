use std::{io::SeekFrom, path::Path};

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

use crate::{
    features::game_log::domain::{GameLogError, LogCursor, LogPage},
    shared::io::domain::IoError,
};

pub(super) async fn read_page(
    path: &Path,
    cursor: LogCursor,
    max_bytes: u64,
) -> Result<LogPage, GameLogError> {
    let io = |err| IoError::with_path(err, path);
    let mut file = File::open(path).await.map_err(io)?;
    let size = file.metadata().await.map_err(io)?.len();

    let (start, end) = match cursor {
        LogCursor::Tail => (size.saturating_sub(max_bytes), size),
        LogCursor::Before(at) => {
            let end = at.min(size);
            (end.saturating_sub(max_bytes), end)
        }
        LogCursor::After(at) => {
            let start = at.min(size);
            (start, start.saturating_add(max_bytes).min(size))
        }
    };

    let mut buf = vec![0u8; usize::try_from(end - start).unwrap_or(0)];
    file.seek(SeekFrom::Start(start)).await.map_err(io)?;
    file.read_exact(&mut buf).await.map_err(io)?;

    // Positions sit on line starts, so only an edge cut by the page size can split a line: drop
    // the cut part (it belongs to the next page) unless that would leave nothing.
    let (mut lo, mut hi) = (0, buf.len());
    let after = matches!(cursor, LogCursor::After(_));
    if after && end < size {
        hi = buf
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(hi, |pos| pos + 1);
    }
    if !after && start > 0 {
        lo = buf
            .iter()
            .position(|&b| b == b'\n')
            .filter(|pos| pos + 1 < buf.len())
            .map_or(0, |pos| pos + 1);
    }

    Ok(LogPage {
        text: String::from_utf8_lossy(&buf[lo..hi]).into_owned(),
        start: start + lo as u64,
        end: start + hi as u64,
        has_more_before: start + lo as u64 > 0,
        has_more_after: start + (hi as u64) < size,
        size,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn file_with(content: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("latest.log");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn numbered(lines: usize) -> Vec<u8> {
        (0..lines)
            .flat_map(|i| format!("line {i:04}\n").into_bytes())
            .collect()
    }

    #[tokio::test]
    async fn should_return_whole_lines_ending_at_the_end_when_reading_the_tail() {
        let (_dir, path) = file_with(&numbered(1000));

        let page = read_page(&path, LogCursor::Tail, 1000).await.unwrap();

        assert!(page.text.starts_with("line "));
        assert!(page.text.ends_with("line 0999\n"));
        assert_eq!(page.end, 10_000);
        assert_eq!(page.size, 10_000);
        assert!(page.has_more_before);
        assert!(!page.has_more_after);
        assert_eq!(page.text.len() as u64, page.end - page.start);
    }

    #[tokio::test]
    async fn should_continue_without_gaps_when_paging_forward_to_the_end() {
        let content = numbered(1000);
        let (_dir, path) = file_with(&content);

        let mut text = String::new();
        let mut at = 0;
        loop {
            let page = read_page(&path, LogCursor::After(at), 1000).await.unwrap();
            assert_eq!(page.start, at);
            text.push_str(&page.text);
            at = page.end;
            if !page.has_more_after {
                break;
            }
        }

        assert_eq!(text.as_bytes(), content.as_slice());
    }

    #[tokio::test]
    async fn should_continue_without_gaps_when_paging_backward_to_the_start() {
        let content = numbered(1000);
        let (_dir, path) = file_with(&content);

        let mut pages = Vec::new();
        let mut page = read_page(&path, LogCursor::Tail, 1000).await.unwrap();
        loop {
            pages.push(page.text.clone());
            if !page.has_more_before {
                break;
            }
            page = read_page(&path, LogCursor::Before(page.start), 1000)
                .await
                .unwrap();
        }

        pages.reverse();
        assert_eq!(pages.concat().as_bytes(), content.as_slice());
    }

    #[tokio::test]
    async fn should_read_nothing_new_when_the_cursor_is_at_the_end() {
        let (_dir, path) = file_with(&numbered(3));

        let page = read_page(&path, LogCursor::After(30), 1000).await.unwrap();

        assert_eq!(page.text, "");
        assert!(!page.has_more_after);
    }

    #[tokio::test]
    async fn should_keep_a_line_longer_than_the_page_when_it_has_no_newline() {
        let (_dir, path) = file_with(&vec![b'a'; 5000]);

        let page = read_page(&path, LogCursor::Tail, 1000).await.unwrap();

        assert_eq!(page.text.len(), 1000);
        assert!(page.has_more_before);
    }
}
