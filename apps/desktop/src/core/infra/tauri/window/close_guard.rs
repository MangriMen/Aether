use aether_core::features::process::ProcessFeature;
use log::{error, warn};
use tauri::{Emitter, Manager, WebviewWindow, WindowEvent};

use crate::core::{CloseConfirmationState, ContainerState, PreventExitState};

/// Sent to the window when the user closes it while a game runs and the close would end the
/// launcher: the frontend asks the user, and destroys the window once they agree.
pub const CLOSE_CONFIRMATION_EVENT: &str = "aether://close-confirmation-requested";

/// Holds the close the user asks for while a game runs, so that the user can be asked first.
///
/// A close the launcher asks for itself (it sets [`PreventExitState`] first, and keeps running)
/// goes through, and so does a hide: neither ends the launcher.
pub fn guard_close_with_running_games<R: tauri::Runtime>(window: &WebviewWindow<R>) {
    let guarded = window.clone();
    window.on_window_event(move |event| {
        let WindowEvent::CloseRequested { api, .. } = event else {
            return;
        };

        let app = guarded.app_handle();
        let Some(container) = app.try_state::<ContainerState>() else {
            return;
        };
        if app.state::<PreventExitState>().is_prevented()
            || !app.state::<CloseConfirmationState>().is_ready()
        {
            return;
        }

        api.prevent_close();

        let window = guarded.clone();
        let games = container.0.running_game_service();
        tauri::async_runtime::spawn(async move {
            // A game that cannot be looked at counts as running: asking once too often costs
            // less than ending the launcher under a game.
            let running = games.any_running().await.unwrap_or_else(|err| {
                warn!("Could not tell whether a game runs: {err}");
                true
            });

            let result = if running {
                window.emit(CLOSE_CONFIRMATION_EVENT, ())
            } else {
                window.destroy()
            };
            if let Err(err) = result {
                error!("Could not handle the close of the window: {err}");
            }
        });
    });
}
