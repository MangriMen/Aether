use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{
    fs::{self, File},
    io::{AsyncBufRead, AsyncRead, AsyncWriteExt, BufReader},
    sync::{mpsc, oneshot},
};

use crate::features::{
    game_log::{
        app::{CaptureGameOutputParams, CaptureHandle, GameOutputCaptureService},
        domain::{Log4jNormalizer, Redactor},
    },
    settings::LocationInfo,
};

/// Console logs of earlier runs kept next to the one being written.
const KEPT_RUNS: usize = 5;
const MAX_LINE_BYTES: usize = 16 * 1024;
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const FILE_PREFIX: &str = "console-";
const FILE_SUFFIX: &str = ".log";
const MAX_BATCH_MESSAGES: usize = 512;

pub struct FsGameOutputCapture {
    location_info: Arc<LocationInfo>,
}

impl FsGameOutputCapture {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }
}

impl GameOutputCaptureService for FsGameOutputCapture {
    fn start(&self, params: CaptureGameOutputParams) -> CaptureHandle {
        let (done_tx, done_rx) = oneshot::channel();
        let dir = self.location_info.game_logs_dir(&params.instance_id);

        tokio::spawn(async move {
            capture(&dir, params).await;
            let _ = done_tx.send(());
        });

        CaptureHandle::new(done_rx)
    }
}

async fn capture(dir: &Path, params: CaptureGameOutputParams) {
    let redactor = Arc::new(Redactor::new(params.secrets));
    let (tx, mut rx) = mpsc::channel::<Vec<String>>(1024);

    if let Some(stdout) = params.stdout {
        tokio::spawn(pump(stdout, redactor.clone(), tx.clone()));
    }
    if let Some(stderr) = params.stderr {
        tokio::spawn(pump(stderr, redactor, tx.clone()));
    }
    drop(tx);

    let mut sink = LogSink::open(dir).await;

    while let Some(first) = rx.recv().await {
        let mut text = join_lines(&first);
        for _ in 0..MAX_BATCH_MESSAGES {
            match rx.try_recv() {
                Ok(more) => text.push_str(&join_lines(&more)),
                Err(_) => break,
            }
        }
        sink.write(&text).await;
    }
    sink.flush().await;
}

fn join_lines(lines: &[String]) -> String {
    let mut text = String::new();
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    text
}

/// A log file that goes away on the first failure while the capture keeps draining.
struct LogSink {
    file: Option<File>,
    written: u64,
}

impl LogSink {
    async fn open(dir: &Path) -> Self {
        let file = match open_run_file(dir).await {
            Ok(file) => Some(file),
            Err(err) => {
                tracing::warn!(
                    "cannot create the game console log in {}: {err}",
                    dir.display()
                );
                None
            }
        };
        Self { file, written: 0 }
    }

    async fn write(&mut self, text: &str) {
        let Some(file) = self.file.as_mut() else {
            return;
        };

        if self.written + text.len() as u64 > MAX_FILE_BYTES {
            let _ = file
                .write_all(b"[console log size limit reached, the rest is not recorded]\n")
                .await;
            self.file = None;
            return;
        }

        match file.write_all(text.as_bytes()).await {
            Ok(()) => self.written += text.len() as u64,
            Err(err) => {
                tracing::warn!("cannot write the game console log: {err}");
                self.file = None;
            }
        }
    }

    async fn flush(&mut self) {
        if let Some(file) = self.file.as_mut() {
            let _ = file.flush().await;
        }
    }
}

async fn open_run_file(dir: &Path) -> std::io::Result<File> {
    fs::create_dir_all(dir).await?;
    prune_old_runs(dir).await;

    let name = format!(
        "{FILE_PREFIX}{}{FILE_SUFFIX}",
        chrono::Local::now().format("%Y%m%d-%H%M%S-%3f")
    );
    File::create(dir.join(name)).await
}

/// Leaves room for the file about to be created. Failures are ignored: a log that cannot be
/// pruned must not stop the game from starting.
async fn prune_old_runs(dir: &Path) {
    let Ok(mut entries) = fs::read_dir(dir).await else {
        return;
    };

    let mut runs: Vec<PathBuf> = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(FILE_PREFIX) && name.ends_with(FILE_SUFFIX) {
            runs.push(entry.path());
        }
    }

    runs.sort();
    let excess = runs.len().saturating_sub(KEPT_RUNS - 1);
    for path in runs.into_iter().take(excess) {
        let _ = fs::remove_file(path).await;
    }
}

async fn pump<R>(stream: R, redactor: Arc<Redactor>, tx: mpsc::Sender<Vec<String>>)
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
    let lines = lines.iter().map(|line| redactor.apply(line)).collect();
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

    #[tokio::test]
    async fn should_prune_the_oldest_runs_when_more_than_the_kept_count_exist() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..7 {
            let name = format!("{FILE_PREFIX}2026010{i}-000000-000{FILE_SUFFIX}");
            std::fs::write(dir.path().join(name), "x").unwrap();
        }
        std::fs::write(dir.path().join("keep.txt"), "x").unwrap();

        prune_old_runs(dir.path()).await;

        let mut names: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names.len(), KEPT_RUNS - 1 + 1);
        assert!(names.contains(&"keep.txt".to_string()));
        assert!(!names.iter().any(|n| n.contains("20260100")));
    }

    #[tokio::test]
    async fn should_write_normalised_redacted_output_and_signal_done_when_the_streams_close() {
        let dir = tempfile::tempdir().unwrap();
        let token = "t".repeat(24);
        let input = format!(
            "<log4j:Event logger=\"x\" timestamp=\"1700000000000\" level=\"INFO\" thread=\"main\">\n<log4j:Message><![CDATA[token {token}]]></log4j:Message>\n</log4j:Event>\nplain\n"
        );

        let (tx, mut rx) = mpsc::channel::<Vec<String>>(8);
        let redactor = Arc::new(Redactor::new([token]));
        pump(input.as_bytes(), redactor, tx).await;
        let mut sink = LogSink::open(dir.path()).await;
        while let Some(lines) = rx.recv().await {
            sink.write(&join_lines(&lines)).await;
        }
        sink.flush().await;

        let file = std::fs::read_dir(dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        let text = std::fs::read_to_string(file.path()).unwrap();
        assert!(text.contains("[main/INFO]: token ***"), "{text}");
        assert!(text.ends_with("plain\n"));
        assert!(!text.contains(&"t".repeat(24)));
    }
}
