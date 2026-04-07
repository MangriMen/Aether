use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Builds and configures the main application window
pub fn build_main_window(
    app_handle: AppHandle,
    transparent: bool,
    visible: bool,
) -> tauri::Result<WebviewWindow> {
    const WINDOW_TITLE: &str = "Aether";
    const DEFAULT_SIZE: (f64, f64) = (1024.0, 640.0);
    const MIN_SIZE: (f64, f64) = (768.0, 480.0);
    const ENTRY_POINT: &str = "index.html";
    const MAIN_WINDOW_TITLE: &str = "main";

    let mut builder = WebviewWindowBuilder::new(
        &app_handle,
        MAIN_WINDOW_TITLE,
        WebviewUrl::App(ENTRY_POINT.into()),
    )
    .title(WINDOW_TITLE)
    .inner_size(DEFAULT_SIZE.0, DEFAULT_SIZE.1)
    .min_inner_size(MIN_SIZE.0, MIN_SIZE.1)
    .decorations(false)
    .focused(true)
    .visible(visible);

    // Apply platform-specific transparency settings
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        builder = builder.transparent(transparent);
    }

    builder.build()
}
