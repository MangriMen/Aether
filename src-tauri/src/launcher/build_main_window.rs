use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub fn build_main_window(
    app_handle: AppHandle,
    transparent: bool,
    visible: bool,
) -> tauri::Result<WebviewWindow> {
    let mut window =
        WebviewWindowBuilder::new(&app_handle, "main", WebviewUrl::App("index.html".into()))
            .title("Aether")
            .inner_size(1024.0, 640.0)
            .min_inner_size(768.0, 480.0)
            .decorations(false)
            .focused(true)
            .visible(visible);

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        window = window.transparent(transparent);
    }

    window.build()
}
