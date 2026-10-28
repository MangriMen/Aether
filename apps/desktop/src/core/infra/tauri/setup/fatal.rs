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
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Warning)
        .set_title("Aether")
        .set_description(message)
        .show();
}

fn show_error(message: &str) {
    error!("{message}");
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Aether")
        .set_description(message)
        .show();
}

/// Logs every panic and, in release, shows it before the process aborts
/// (`panic = "abort"` gives no other chance to tell the user).
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let message = describe_panic(info);
        error!("{message}");

        if cfg!(not(debug_assertions)) {
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

    #[test]
    fn should_include_reason_and_location_in_panic_message() {
        let message = format_panic("boom", Some(("src/lib.rs", 7)));

        assert_eq!(message, "Aether crashed unexpectedly: boom (src/lib.rs:7)");
    }

    #[test]
    fn should_omit_location_when_unknown() {
        assert_eq!(
            format_panic("boom", None),
            "Aether crashed unexpectedly: boom"
        );
    }
}
