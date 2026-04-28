use async_trait::async_trait;
use tauri::{Manager, State};

use crate::{
    core::{PreventExitState, WindowError, WindowLabel, WindowManager},
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

    async fn create_window(
        &self,
        label: WindowLabel,
        transparent: bool,
        effect: WindowEffect,
    ) -> Result<(), WindowError> {
        if self.get_window(label).is_err() {
            build_main_window(&self.app_handle, label.as_ref(), transparent, false)?;
        }

        self.apply_visual_effect(label, effect).await?;

        if label == WindowLabel::Main {
            self.get_prevent_exit_state().set_prevented(false);
        }

        Ok(())
    }
}
