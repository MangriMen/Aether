use aether_core::shared::read_async;
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
        format!("{}://{}", ASSET_PROTOCOL, path)
    }
}

pub async fn handle_asset_request<R: Runtime>(
    app_handle: AppHandle<R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: tauri::UriSchemeResponder,
) {
    let location_info = app_handle.state::<LocationInfoState>().inner().clone();
    let assets_dir = location_info.assets_cache_dir();

    let uri = request.uri().path();
    let decoded_uri = percent_encoding::percent_decode_str(uri).decode_utf8_lossy();
    let asset_id = decoded_uri.trim_start_matches('/');

    let file_path = assets_dir.join(asset_id);

    if file_path.exists() && file_path.starts_with(&assets_dir) {
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
    } else {
        respond_with_code(responder, StatusCode::NOT_FOUND);
    }
}

fn respond_with_code(responder: tauri::UriSchemeResponder, status: StatusCode) {
    responder.respond(Response::builder().status(status).body(Vec::new()).unwrap());
}
