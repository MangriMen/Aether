use tauri::{AppHandle, Manager, Runtime};

use crate::core::{
    AppSettingsStorageState, PreventExitState, WindowLabel, WindowManager, WindowManagerState,
};
use crate::features::settings::AppSettingsStorage;

/// Brings the running launcher's window forward when a second copy is started.
///
/// The second copy exits before it opens the database. Startup creates the window itself, so a
/// missing window is only recreated when exit is being prevented: the window was closed for a
/// game (or a failed recreation) and the launcher is otherwise unreachable.
pub fn focus_main_window<R: Runtime>(app: &AppHandle<R>) {
    let window = app.get_webview_window(WindowLabel::Main.as_ref());
    let exit_prevented = app
        .try_state::<PreventExitState>()
        .is_some_and(|state| state.is_prevented());

    let Some(window) = window else {
        if should_restore_window(false, exit_prevented) {
            restore_main_window(app);
        }
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

fn should_restore_window(window_exists: bool, exit_prevented: bool) -> bool {
    !window_exists && exit_prevented
}

fn restore_main_window<R: Runtime>(app: &AppHandle<R>) {
    let (Some(settings_storage), Some(window_manager)) = (
        app.try_state::<AppSettingsStorageState>(),
        app.try_state::<WindowManagerState<R>>(),
    ) else {
        return;
    };
    let settings_storage = settings_storage.inner().clone();
    let window_manager = window_manager.inner().clone();

    tauri::async_runtime::spawn(async move {
        let settings = match settings_storage.get().await {
            Ok(settings) => settings,
            Err(err) => {
                tracing::warn!("Failed to read settings to restore the main window: {err}");
                return;
            }
        };

        if let Err(err) = window_manager
            .create_window(
                WindowLabel::Main,
                settings.transparent,
                settings.window_effect,
            )
            .await
        {
            tracing::warn!("Failed to restore the main window: {err}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::closed_for_game(false, true, true)]
    #[case::startup_in_progress(false, false, false)]
    #[case::window_present(true, true, false)]
    #[case::window_present_normal(true, false, false)]
    fn should_restore_only_a_missing_window_while_exit_is_prevented(
        #[case] window_exists: bool,
        #[case] exit_prevented: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(
            should_restore_window(window_exists, exit_prevented),
            expected
        );
    }
}
