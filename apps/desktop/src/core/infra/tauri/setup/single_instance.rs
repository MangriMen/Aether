use tauri::{AppHandle, Manager, Runtime};

use crate::core::WindowLabel;

/// Brings the running launcher's window forward when a second copy is started.
///
/// The second copy exits before it opens the database. It does not create a missing window:
/// during startup the first copy creates it itself, and while a game runs with "close on
/// launch" the window comes back when the game ends.
pub fn focus_main_window<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(WindowLabel::Main.as_ref()) else {
        return;
    };

    if let Err(err) = window
        .show()
        .and_then(|()| window.unminimize())
        .and_then(|()| window.set_focus())
    {
        tracing::warn!("Failed to bring the main window forward: {err}");
    }
}
