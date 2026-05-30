use std::sync::Arc;

use aether_core::features::events::{ProcessEvent, ProcessEventType};
use log::error;

use crate::{
    core::{WindowLabel, WindowManager},
    features::settings::{ActionOnInstanceLaunch, AppSettingsStorage},
};

pub struct InstanceLaunchListener<ASS: AppSettingsStorage, WM: WindowManager> {
    app_settings_storage: Arc<ASS>,
    window_manager: Arc<WM>,
}

impl<ASS: AppSettingsStorage, WM: WindowManager> InstanceLaunchListener<ASS, WM> {
    pub fn new(app_settings_storage: Arc<ASS>, window_manager: Arc<WM>) -> Self {
        Self {
            app_settings_storage,
            window_manager,
        }
    }

    pub async fn on_process_event(&self, event: ProcessEvent) {
        match event.event {
            ProcessEventType::Launched => self
                .handle_instance_launch()
                .await
                .unwrap_or_else(|err| error!("Failed to handle instance launch: {err}")),

            ProcessEventType::Finished => self
                .handle_instance_finish()
                .await
                .unwrap_or_else(|err| error!("Failed to handle instance finish: {err}")),
        }
    }

    async fn handle_instance_launch(&self) -> Result<(), crate::Error> {
        let app_settings = self.app_settings_storage.get().await?;

        match app_settings.action_on_instance_launch {
            ActionOnInstanceLaunch::Hide => {
                self.window_manager.hide_window(WindowLabel::Main).await?;
            }

            ActionOnInstanceLaunch::Close => {
                self.window_manager.close_window(WindowLabel::Main).await?;
            }

            ActionOnInstanceLaunch::Nothing => {}
        }

        Ok(())
    }

    async fn handle_instance_finish(&self) -> Result<(), crate::Error> {
        let app_settings = self.app_settings_storage.get().await?;

        match app_settings.action_on_instance_launch {
            ActionOnInstanceLaunch::Hide => {
                self.window_manager.show_window(WindowLabel::Main).await?;
            }

            ActionOnInstanceLaunch::Close => {
                self.window_manager
                    .create_window(
                        WindowLabel::Main,
                        app_settings.transparent,
                        app_settings.window_effect,
                    )
                    .await?;
            }

            ActionOnInstanceLaunch::Nothing => {}
        }

        Ok(())
    }
}
