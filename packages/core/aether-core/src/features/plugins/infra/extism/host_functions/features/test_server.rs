//! A minimal HTTP/1.1 server for the network host-function tests.
//!
//! Each connection carries exactly one request: the server reads it, answers with whatever the
//! handler returns for its path, and closes. It counts connections and keeps every raw request,
//! so a test can prove that a host was never contacted, or what exactly it received.
//!
//! It binds to a chosen loopback address, so two servers on `127.0.0.1` and `127.0.0.2` are two
//! different hosts as far as `allowed_hosts`, the address filter and the origin are concerned.
//! Linux and Windows serve all of `127.0.0.0/8`; macOS needs `127.0.0.2` aliased on `lo0`
//! (`sudo ifconfig lo0 alias 127.0.0.2`) before the second server can bind.

use std::{
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

type Handler = dyn Fn(&str) -> Vec<u8> + Send + Sync;

pub(crate) struct TestServer {
    addr: SocketAddr,
    connections: Arc<AtomicUsize>,
    requests: Arc<Mutex<Vec<String>>>,
}

impl TestServer {
    /// Serve on `ip` (a loopback address), answering each request with `respond(path)`.
    pub(crate) async fn start(
        ip: &str,
        respond: impl Fn(&str) -> Vec<u8> + Send + Sync + 'static,
    ) -> Self {
        let listener = tokio::net::TcpListener::bind((ip, 0))
            .await
            .unwrap_or_else(|err| panic!("test server should bind to {ip}: {err}"));
        let addr = listener.local_addr().expect("test server should have addr");
        let connections = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let respond: Arc<Handler> = Arc::new(respond);

        let (counter, log) = (connections.clone(), requests.clone());
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(answer(stream, respond.clone(), log.clone()));
            }
        });

        Self {
            addr,
            connections,
            requests,
        }
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    /// How many connections reached the server so far.
    pub(crate) fn connections(&self) -> usize {
        self.connections.load(Ordering::SeqCst)
    }

    /// Every request received so far, raw, lowercased for header matching.
    pub(crate) fn requests(&self) -> Vec<String> {
        self.requests.lock().expect("request log").clone()
    }
}

async fn answer(
    mut stream: tokio::net::TcpStream,
    respond: Arc<Handler>,
    log: Arc<Mutex<Vec<String>>>,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // Read the whole request, body included: closing with unread data would reset the
    // connection under a client that is still writing.
    let mut received = Vec::new();
    let mut buf = [0_u8; 1024];
    loop {
        let text = String::from_utf8_lossy(&received).to_ascii_lowercase();
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let expected = head
                .lines()
                .find_map(|line| line.strip_prefix("content-length: "))
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or(0);
            if body.len() >= expected {
                break;
            }
        }
        match stream.read(&mut buf).await {
            Ok(0) | Err(_) => return,
            Ok(n) => received.extend_from_slice(&buf[..n]),
        }
    }

    let request = String::from_utf8_lossy(&received).into_owned();
    let path = request
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned();
    log.lock()
        .expect("request log")
        .push(request.to_ascii_lowercase());

    let _ = stream.write_all(&respond(&path)).await;
    let _ = stream.flush().await;
    let _ = stream.shutdown().await;
}

/// `200 OK` with `body`.
pub(crate) fn ok(body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

/// A redirect with the given status and `Location`, and a small body that must never be read
/// as the result.
pub(crate) fn redirect(status: u16, location: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status} Redirect\r\nLocation: {location}\r\nContent-Length: 5\r\nConnection: close\r\n\r\nmoved"
    )
    .into_bytes()
}

/// A plain status with no body.
pub(crate) fn status(status: u16) -> Vec<u8> {
    format!("HTTP/1.1 {status} Status\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
        .into_bytes()
}
