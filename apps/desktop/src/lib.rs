pub mod core;
pub mod features;
pub mod shared;

pub(crate) use core::{Error, FrontendResult, Result};

#[cfg(test)]
mod csp_tests {
    #[test]
    fn should_not_allow_plaintext_http_images_in_csp() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let csp = conf["app"]["security"]["csp"].as_str().unwrap();
        let img_src = csp
            .split(';')
            .map(str::trim)
            .find(|d| d.starts_with("img-src"))
            .unwrap();

        assert!(!img_src.split_whitespace().any(|s| s == "http://*"));
        assert!(img_src.split_whitespace().any(|s| s == "https://*"));
    }

    #[test]
    fn should_allow_https_media_but_not_plaintext_http_in_csp() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let csp = conf["app"]["security"]["csp"].as_str().unwrap();
        let media_src = csp
            .split(';')
            .map(str::trim)
            .find(|d| d.starts_with("media-src"))
            .expect("without media-src, Markdown <video> falls back to default-src and is blocked");

        assert!(!media_src.split_whitespace().any(|s| s == "http://*"));
        assert!(media_src.split_whitespace().any(|s| s == "https://*"));
    }
}
