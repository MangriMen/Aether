use std::time::Duration;

use async_trait::async_trait;
use log::warn;
use tauri::{Manager, State};
use tauri_plugin_window_state::AppHandleExt;

use crate::{
    core::{
        PreventExitState, WindowError, WindowLabel, WindowManager, get_main_window_state_flags,
    },
    features::settings::WindowEffect,
};

use super::build_main_window::build_main_window;
use super::window_effect::apply_effect_to_window;

pub struct TauriWindowManager<R: tauri::Runtime> {
    app_handle: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> TauriWindowManager<R> {
    pub fn new(app_handle: tauri::AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

impl<R: tauri::Runtime> TauriWindowManager<R> {
    pub fn get_window(&self, label: WindowLabel) -> Result<tauri::WebviewWindow<R>, WindowError> {
        let label_ref = label.as_ref();

        let window = self.app_handle.get_webview_window(label_ref);

        let Some(window) = window else {
            return Err(WindowError::NotFound { label });
        };

        Ok(window)
    }

    fn get_prevent_exit_state(&self) -> State<'_, PreventExitState> {
        self.app_handle.state::<PreventExitState>()
    }

    async fn window_close_polling(&self, label: WindowLabel) {
        let one_tick_duration = Duration::from_millis(50);

        let minimal_duration = Duration::from_millis(200);
        let full_duration = Duration::from_secs(1);

        let mut current_duration = Duration::ZERO;

        while (self.get_window(label).is_ok() && current_duration < full_duration)
            || current_duration < minimal_duration
        {
            tokio::time::sleep(one_tick_duration).await;
            current_duration += one_tick_duration;
        }
    }

    fn save_window_state(&self) {
        if let Err(err) = self
            .app_handle
            .save_window_state(get_main_window_state_flags())
        {
            warn!("Failed to save window state: {err}");
        }
    }
}

#[async_trait]
impl<R: tauri::Runtime> WindowManager for TauriWindowManager<R> {
    async fn apply_visual_effect(
        &self,
        label: WindowLabel,
        effect: WindowEffect,
    ) -> Result<(), WindowError> {
        let window = self.get_window(label)?;

        Ok(apply_effect_to_window(&window, effect.into())?)
    }

    async fn hide_window(&self, label: WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;

        Ok(window.hide()?)
    }

    async fn show_window(&self, label: WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;

        Ok(window.show()?)
    }

    async fn close_window(&self, label: WindowLabel) -> Result<(), WindowError> {
        if label == WindowLabel::Main {
            self.get_prevent_exit_state().set_prevented(true);
        }

        let window = self.get_window(label)?;

        Ok(window.close()?)
    }

    async fn close_window_and_wait(&self, label: WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;

        if label == WindowLabel::Main {
            self.get_prevent_exit_state().set_prevented(true);
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

        window.on_window_event(move |event| {
            if matches!(event, tauri::WindowEvent::Destroyed)
                && let Ok(mut guard) = tx.lock()
                && let Some(sender) = guard.take()
            {
                let _ = sender.send(());
            }
        });

        self.save_window_state();
        window.close()?;

        if tokio::time::timeout(Duration::from_secs(1), rx)
            .await
            .is_err()
        {
            warn!(
                "No window destruction event was received for window {}",
                label.as_ref()
            );
        }

        if tokio::time::timeout(Duration::from_secs(1), self.window_close_polling(label))
            .await
            .is_err()
        {
            warn!("Window {} was not closed", label.as_ref());
        }

        Ok(())
    }

    async fn create_window(
        &self,
        label: WindowLabel,
        transparent: bool,
        effect: WindowEffect,
    ) -> Result<(), WindowError> {
        if self.get_window(label).is_ok() {
            return Err(WindowError::AlreadyExists { label });
        }

        let window = build_main_window(&self.app_handle, label.as_ref(), transparent, false)?;

        if label == WindowLabel::Main {
            self.get_prevent_exit_state().set_prevented(false);
        }

        apply_effect_to_window(&window, effect.into())?;

        Ok(())
    }
}
