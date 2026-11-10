use std::sync::Arc;

use tokio::{
    io::{AsyncBufRead, AsyncRead, BufReader},
    sync::mpsc,
};

use crate::features::game_log::domain::{Log4jNormalizer, Redactor};

const MAX_LINE_BYTES: usize = 16 * 1024;

pub(super) async fn pump<R>(stream: R, redactor: Arc<Redactor>, tx: mpsc::Sender<Vec<String>>)
where
    R: AsyncRead + Unpin,
{
    let mut reader = BufReader::new(stream);
    let mut normalizer = Log4jNormalizer::new();
    let mut buf = Vec::new();

    while let Ok(Some(line)) = read_capped_line(&mut reader, &mut buf).await {
        let out = normalizer.push_line(&line);
        if !send(&tx, &redactor, out).await {
            return;
        }
    }

    let rest = normalizer.finish();
    send(&tx, &redactor, rest).await;
}

async fn send(tx: &mpsc::Sender<Vec<String>>, redactor: &Redactor, lines: Vec<String>) -> bool {
    if lines.is_empty() {
        return true;
    }
    let lines = lines
        .iter()
        .map(|line| redactor.apply(line).into_owned())
        .collect();
    tx.send(lines).await.is_ok()
}

/// Reads one line without letting a newline-less stream grow the buffer: bytes past the cap
/// are dropped up to the line end. Strips the `\r` of Windows line ends and decodes lossily
/// per complete line, so a multi-byte character is never cut at a chunk edge.
async fn read_capped_line<R>(reader: &mut R, buf: &mut Vec<u8>) -> std::io::Result<Option<String>>
where
    R: AsyncBufRead + Unpin,
{
    use tokio::io::AsyncBufReadExt;

    buf.clear();
    let mut truncated = false;
    let mut read_any = false;

    loop {
        let chunk = reader.fill_buf().await?;
        if chunk.is_empty() {
            if !read_any {
                return Ok(None);
            }
            break;
        }
        read_any = true;

        let (take, found_newline) = match chunk.iter().position(|&b| b == b'\n') {
            Some(pos) => (pos + 1, true),
            None => (chunk.len(), false),
        };

        let room = MAX_LINE_BYTES.saturating_sub(buf.len());
        let keep = take.min(room);
        buf.extend_from_slice(&chunk[..keep]);
        if keep < take {
            truncated = true;
        }
        reader.consume(take);

        if found_newline {
            break;
        }
    }

    while matches!(buf.last(), Some(b'\n' | b'\r')) {
        buf.pop();
    }
    let mut line = String::from_utf8_lossy(buf).into_owned();
    if truncated {
        line.push_str(" [line truncated]");
    }
    Ok(Some(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn lines_of(input: &[u8]) -> Vec<String> {
        let mut reader = BufReader::new(input);
        let mut buf = Vec::new();
        let mut out = Vec::new();
        while let Some(line) = read_capped_line(&mut reader, &mut buf).await.unwrap() {
            out.push(line);
        }
        out
    }

    #[tokio::test]
    async fn should_strip_carriage_returns_when_lines_end_with_crlf() {
        assert_eq!(lines_of(b"one\r\ntwo\r\n").await, ["one", "two"]);
    }

    #[tokio::test]
    async fn should_keep_the_last_line_when_it_has_no_newline() {
        assert_eq!(lines_of(b"one\ntail").await, ["one", "tail"]);
    }

    #[tokio::test]
    async fn should_replace_invalid_utf8_when_a_line_holds_native_bytes() {
        let out = lines_of(b"bad \xff byte\n").await;

        assert_eq!(out, ["bad \u{fffd} byte"]);
    }

    #[tokio::test]
    async fn should_truncate_a_line_when_it_exceeds_the_cap() {
        let mut input = vec![b'a'; MAX_LINE_BYTES * 2];
        input.extend_from_slice(b"\nnext\n");

        let out = lines_of(&input).await;

        assert_eq!(out.len(), 2);
        assert!(out[0].ends_with("[line truncated]"));
        assert_eq!(out[1], "next");
    }
}
