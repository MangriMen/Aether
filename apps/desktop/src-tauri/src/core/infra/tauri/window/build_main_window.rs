#![allow(clippy::needless_pass_by_value)]
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const MAIN_WINDOW_LABEL: &str = "main";

const WINDOW_TITLE: &str = "Aether";

const DEFAULT_SIZE: (f64, f64) = (1024.0, 640.0);
const MIN_SIZE: (f64, f64) = (768.0, 480.0);

/// Builds and configures the main application window
pub fn build_main_window<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    transparent: bool,
    visible: bool,
) -> tauri::Result<WebviewWindow<R>> {
    const ENTRY_POINT: &str = "index.html";

    let mut builder = WebviewWindowBuilder::new(
        &app_handle,
        MAIN_WINDOW_LABEL,
        WebviewUrl::App(ENTRY_POINT.into()),
    )
    .title(WINDOW_TITLE)
    .inner_size(DEFAULT_SIZE.0, DEFAULT_SIZE.1)
    .min_inner_size(MIN_SIZE.0, MIN_SIZE.1)
    .decorations(false)
    .focused(true)
    .visible(visible);

    // Apply platform-specific transparency settings
    #[cfg(target_os = "windows")]
    {
        // Позволяет кликать "сквозь" прозрачные области, если нужно
        builder = builder.transparent(transparent).shadow(true);
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder.transparent(transparent).shadow(true);
    }

    #[cfg(target_os = "linux")]
    {
        builder = builder.transparent(transparent);
    }

    let window = builder.build()?;

    Ok(window)
}
