use std::path::{Path, PathBuf};

use aether_core::shared::io::infra::{canonicalize, read_async};
use tauri::{
    AppHandle, Manager, Runtime,
    http::{Response, StatusCode, header::ACCESS_CONTROL_ALLOW_ORIGIN},
};

use crate::core::LocationInfoState;

pub const ASSET_PROTOCOL: &str = "aether-asset";

pub fn format_asset_url(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("http://{ASSET_PROTOCOL}.localhost/{path}")
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("{ASSET_PROTOCOL}://{path}")
    }
}

pub async fn handle_asset_request<R: Runtime>(
    app_handle: AppHandle<R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: tauri::UriSchemeResponder,
) {
    let location_info = app_handle.state::<LocationInfoState>().inner().clone();
    let assets_dir = location_info.assets_cache_dir();

    let Some(file_path) = resolve_asset_path(&assets_dir, request.uri().path()) else {
        respond_with_code(responder, StatusCode::NOT_FOUND);
        return;
    };

    match read_async(&file_path).await {
        Ok(content) => {
            let mime = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            let response = Response::builder()
                .header("Content-Type", mime)
                .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header("Cache-Control", "public, max-age=31536000, immutable")
                .body(content)
                .unwrap();

            responder.respond(response);
        }
        Err(_) => {
            respond_with_code(responder, StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

/// The file of the asset store a request names, or `None` for anything else. The check is made
/// on the canonical paths after the percent-decoding: `..%2f` or a symlink would otherwise
/// walk out of the store.
fn resolve_asset_path(assets_dir: &Path, uri_path: &str) -> Option<PathBuf> {
    let decoded_uri = percent_encoding::percent_decode_str(uri_path).decode_utf8_lossy();
    let asset_id = decoded_uri.trim_start_matches('/');

    let canonical_dir = canonicalize(assets_dir).ok()?;
    let canonical_file = canonicalize(assets_dir.join(asset_id)).ok()?;

    (canonical_file.starts_with(&canonical_dir) && canonical_file.is_file())
        .then_some(canonical_file)
}

fn respond_with_code(responder: tauri::UriSchemeResponder, status: StatusCode) {
    responder.respond(Response::builder().status(status).body(Vec::new()).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let dir =
                std::env::temp_dir().join(format!("aether-protocols-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(dir.join("assets")).unwrap();
            Self(dir)
        }

        fn assets(&self) -> PathBuf {
            self.0.join("assets")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn should_resolve_the_asset_when_it_is_in_the_store() {
        let temp = TempDir::new();
        std::fs::write(temp.assets().join("icon.png"), b"png").unwrap();

        let resolved = resolve_asset_path(&temp.assets(), "/icon.png").unwrap();

        assert!(resolved.ends_with("icon.png"));
    }

    #[test]
    fn should_refuse_the_asset_when_it_does_not_exist() {
        let temp = TempDir::new();

        assert_eq!(resolve_asset_path(&temp.assets(), "/missing.png"), None);
    }

    #[test]
    fn should_refuse_the_asset_when_the_path_leaves_the_store() {
        let temp = TempDir::new();
        std::fs::write(temp.0.join("secret.txt"), b"secret").unwrap();

        for uri in [
            "/../secret.txt",
            "/..%2fsecret.txt",
            "/%2e%2e/secret.txt",
            "/..%5csecret.txt",
        ] {
            assert_eq!(
                resolve_asset_path(&temp.assets(), uri),
                None,
                "{uri} must not leave the store"
            );
        }
    }
}
