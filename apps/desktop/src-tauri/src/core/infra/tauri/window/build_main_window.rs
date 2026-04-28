#![allow(clippy::needless_pass_by_value)]
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_window_state::StateFlags;

const WINDOW_TITLE: &str = "Aether";

const DEFAULT_SIZE: (f64, f64) = (1024.0, 640.0);
const MIN_SIZE: (f64, f64) = (768.0, 480.0);

pub fn get_main_window_state_flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED
}

/// Builds and configures the main application window
pub fn build_main_window<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    label: &str,
    transparent: bool,
    visible: bool,
) -> tauri::Result<WebviewWindow<R>> {
    const ENTRY_POINT: &str = "index.html";

    let mut builder =
        WebviewWindowBuilder::new(app_handle, label, WebviewUrl::App(ENTRY_POINT.into()))
            .title(WINDOW_TITLE)
            .inner_size(DEFAULT_SIZE.0, DEFAULT_SIZE.1)
            .min_inner_size(MIN_SIZE.0, MIN_SIZE.1)
            .decorations(false)
            .focused(true)
            .visible(visible);

    // Apply platform-specific transparency settings
    #[cfg(target_os = "windows")]
    {
        builder = builder.transparent(transparent).shadow(true);
    }

    #[cfg(target_os = "macos")]
    {
        #[cfg(feature = "macos-private-api")]
        {
            builder = builder.transparent(transparent).shadow(true);
        }
    }

    #[cfg(target_os = "linux")]
    {
        builder = builder.transparent(transparent);
    }

    let window = builder.build()?;

    Ok(window)
}
