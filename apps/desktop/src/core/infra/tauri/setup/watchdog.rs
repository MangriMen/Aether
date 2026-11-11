use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tauri::{Manager, WebviewWindow};

use super::fatal::exit_with_error;

const WINDOW_VISIBLE_WATCH_DOG_TIMEOUT: Duration = Duration::from_secs(30);
const WINDOW_VISIBLE_WATCH_DOG_RETRY_DELAY: Duration = Duration::from_millis(500);

pub async fn run_window_is_visible_watch_dog<R: tauri::Runtime>(window: WebviewWindow<R>) {
    let has_been_shown = Arc::new(AtomicBool::new(false));

    if window.is_visible().unwrap_or(false) {
        return;
    }

    let has_been_shown_cloned = has_been_shown.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(is_focused) = event
            && *is_focused
        {
            has_been_shown_cloned.store(true, Ordering::Relaxed);
        }
    });

    let start_time = std::time::Instant::now();

    while start_time.elapsed() < WINDOW_VISIBLE_WATCH_DOG_TIMEOUT {
        if has_been_shown.load(Ordering::Relaxed) {
            return;
        }

        if let Ok(true) = window.is_visible() {
            has_been_shown.store(true, Ordering::Relaxed);
            return;
        }

        tokio::time::sleep(WINDOW_VISIBLE_WATCH_DOG_RETRY_DELAY).await;
    }

    if !has_been_shown.load(Ordering::Relaxed)
        && let Ok(false) = window.is_visible()
    {
        let log_dir = window.app_handle().path().app_log_dir().ok();
        exit_with_error(&timeout_message(
            WINDOW_VISIBLE_WATCH_DOG_TIMEOUT,
            WEBVIEW_HINT,
            log_dir.as_deref(),
        ));
    }
}

#[cfg(target_os = "windows")]
const WEBVIEW_HINT: &str = "Make sure the Microsoft Edge WebView2 runtime is installed and that                             antivirus software is not blocking it.";
#[cfg(target_os = "linux")]
const WEBVIEW_HINT: &str = "Make sure WebKitGTK is installed and that a graphical session is                             available.";
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
const WEBVIEW_HINT: &str = "Make sure the system web view is installed and working.";

fn timeout_message(timeout: Duration, webview_hint: &str, log_dir: Option<&Path>) -> String {
    let message = format!(
        "Aether's window did not appear within {} seconds. {webview_hint}",
        timeout.as_secs()
    );

    match log_dir {
        Some(log_dir) => format!("{message}\n\nLogs: {}", log_dir.display()),
        None => message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_name_timeout_and_hint_when_log_dir_is_unknown() {
        let message = timeout_message(Duration::from_secs(30), "Check the web view.", None);

        assert_eq!(
            message,
            "Aether's window did not appear within 30 seconds. Check the web view."
        );
    }

    #[test]
    fn should_point_to_log_dir_when_it_is_known() {
        let message = timeout_message(
            Duration::from_secs(30),
            "Check the web view.",
            Some(Path::new("/logs")),
        );

        assert!(message.ends_with(&format!("Logs: {}", Path::new("/logs").display())));
    }
}
