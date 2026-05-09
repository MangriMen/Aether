use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tauri::WebviewWindow;

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
        panic!("Frontend initialization timeout. Window was never shown or focused.");
    }
}
