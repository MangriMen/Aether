use std::panic::PanicHookInfo;

use log::error;

/// Reports a failure that happens before any window exists, then ends the process.
///
/// A release build has no console (`windows_subsystem = "windows"`), so without
/// the dialog the app would just vanish.
pub fn exit_with_error(message: &str) -> ! {
    show_error(message);
    std::process::exit(1)
}

/// Tells the user about a problem the launcher survives. Blocks until the dialog is closed.
pub fn show_warning(message: &str) {
    tracing::warn!("{message}");
    show_dialog(rfd::MessageLevel::Warning, message);
}

fn show_error(message: &str) {
    error!("{message}");
    show_dialog(rfd::MessageLevel::Error, message);
}

fn show_dialog(level: rfd::MessageLevel, message: &str) {
    #[cfg(target_os = "linux")]
    if show_on_gtk_main_thread(level, message) {
        return;
    }

    rfd::MessageDialog::new()
        .set_level(level)
        .set_title("Aether")
        .set_description(message)
        .show();
}

/// `setup` runs on the GTK main thread inside the event loop, which owns the default main
/// context. The blocking rfd dialog queues its work on that context and waits for it, so called
/// from here it never appears and the process hangs. Running the dialog's own loop on this
/// thread is the only way to show it.
#[cfg(target_os = "linux")]
fn show_on_gtk_main_thread(level: rfd::MessageLevel, message: &str) -> bool {
    use gtk::prelude::*;

    if !gtk::is_initialized_main_thread() {
        return false;
    }

    let message_type = match level {
        rfd::MessageLevel::Error => gtk::MessageType::Error,
        rfd::MessageLevel::Warning => gtk::MessageType::Warning,
        rfd::MessageLevel::Info => gtk::MessageType::Info,
    };

    let dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::MODAL,
        message_type,
        gtk::ButtonsType::Ok,
        "Aether",
    );
    dialog.set_title("Aether");
    dialog.set_secondary_text(Some(message));
    dialog.run();
    dialog.close();

    true
}

/// Without a display GTK cannot initialize in rfd's thread, and the blocking dialog would wait
/// for it forever, so a panic would hang instead of ending the process.
fn can_show_dialog() -> bool {
    #[cfg(target_os = "linux")]
    if !gtk::is_initialized_main_thread()
        && std::env::var_os("DISPLAY").is_none()
        && std::env::var_os("WAYLAND_DISPLAY").is_none()
    {
        return false;
    }

    true
}

/// Logs every panic and, in release, shows it before the process aborts
/// (`panic = "abort"` gives no other chance to tell the user).
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let message = describe_panic(info);
        error!("{message}");

        if cfg!(not(debug_assertions)) && can_show_dialog() {
            show_error(&message);
        }

        previous(info);
    }));
}

fn describe_panic(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    let reason = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown reason");

    format_panic(reason, info.location().map(|l| (l.file(), l.line())))
}

fn format_panic(reason: &str, location: Option<(&str, u32)>) -> String {
    match location {
        Some((file, line)) => format!("Aether crashed unexpectedly: {reason} ({file}:{line})"),
        None => format!("Aether crashed unexpectedly: {reason}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::with_location("boom", Some(("src/lib.rs", 7)), "Aether crashed unexpectedly: boom (src/lib.rs:7)")]
    #[case::unknown_location("boom", None, "Aether crashed unexpectedly: boom")]
    fn should_format_panic_message(
        #[case] reason: &str,
        #[case] location: Option<(&str, u32)>,
        #[case] expected: &str,
    ) {
        assert_eq!(format_panic(reason, location), expected);
    }
}
