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

            ProcessEventType::Finished if !event.adopted => self
                .handle_instance_finish()
                .await
                .unwrap_or_else(|err| error!("Failed to handle instance finish: {err}")),

            // A game an earlier launcher started: this launcher did not hide or close its
            // window for it, so its end has nothing to bring back, and the window the user just
            // opened stays as it is. A failed post-exit hook is shown by the frontend.
            ProcessEventType::Adopted
            | ProcessEventType::Finished
            | ProcessEventType::PostExitHookFailed(_) => {}
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use aether_core::features::instance::InstanceId;
    use async_trait::async_trait;
    use rstest::rstest;
    use uuid::Uuid;

    use super::*;
    use crate::core::WindowError;
    use crate::features::settings::{AppSettings, AppSettingsError, WindowEffect};

    struct Settings(ActionOnInstanceLaunch);

    #[async_trait]
    impl AppSettingsStorage for Settings {
        async fn get(&self) -> Result<AppSettings, AppSettingsError> {
            Ok(AppSettings {
                action_on_instance_launch: self.0,
                is_actual_transparent: false,
                transparent: false,
                window_effect: WindowEffect::Off,
            })
        }

        async fn upsert(&self, _settings: AppSettings) -> Result<(), AppSettingsError> {
            unimplemented!()
        }
    }

    /// Notes what was done to the windows.
    #[derive(Default)]
    struct Windows(Mutex<Vec<&'static str>>);

    impl Windows {
        fn note(&self, what: &'static str) {
            self.0.lock().unwrap().push(what);
        }
    }

    #[async_trait]
    impl WindowManager for Windows {
        async fn apply_visual_effect(
            &self,
            _label: WindowLabel,
            _effect: WindowEffect,
        ) -> Result<(), WindowError> {
            unimplemented!()
        }

        async fn hide_window(&self, _label: WindowLabel) -> Result<(), WindowError> {
            self.note("hide");
            Ok(())
        }

        async fn show_window(&self, _label: WindowLabel) -> Result<(), WindowError> {
            self.note("show");
            Ok(())
        }

        async fn close_window(&self, _label: WindowLabel) -> Result<(), WindowError> {
            self.note("close");
            Ok(())
        }

        async fn close_window_and_wait(&self, _label: WindowLabel) -> Result<(), WindowError> {
            unimplemented!()
        }

        async fn create_window(
            &self,
            _label: WindowLabel,
            _transparent: bool,
            _window_effect: WindowEffect,
        ) -> Result<(), WindowError> {
            self.note("create");
            Ok(())
        }
    }

    fn event(event: ProcessEventType, adopted: bool) -> ProcessEvent {
        ProcessEvent {
            instance_id: InstanceId::try_from("test-instance").unwrap(),
            process_id: Uuid::new_v4(),
            event,
            message: String::new(),
            exit_code: None,
            crashed: false,
            adopted,
        }
    }

    async fn windows_after(
        action: ActionOnInstanceLaunch,
        event: ProcessEvent,
    ) -> Vec<&'static str> {
        let windows = Arc::new(Windows::default());
        let listener = InstanceLaunchListener::new(Arc::new(Settings(action)), windows.clone());

        listener.on_process_event(event).await;

        windows.0.lock().unwrap().clone()
    }

    #[rstest]
    #[case::hide(ActionOnInstanceLaunch::Hide)]
    #[case::close(ActionOnInstanceLaunch::Close)]
    #[case::nothing(ActionOnInstanceLaunch::Nothing)]
    #[tokio::test]
    async fn should_leave_the_windows_alone_when_a_game_is_taken_over(
        #[case] action: ActionOnInstanceLaunch,
    ) {
        let done = windows_after(action, event(ProcessEventType::Adopted, true)).await;

        assert!(done.is_empty(), "{done:?}");
    }

    #[rstest]
    #[case::hide(ActionOnInstanceLaunch::Hide)]
    #[case::close(ActionOnInstanceLaunch::Close)]
    #[case::nothing(ActionOnInstanceLaunch::Nothing)]
    #[tokio::test]
    async fn should_leave_the_windows_alone_when_a_taken_over_game_ends(
        #[case] action: ActionOnInstanceLaunch,
    ) {
        let done = windows_after(action, event(ProcessEventType::Finished, true)).await;

        assert!(done.is_empty(), "{done:?}");
    }

    #[rstest]
    #[case::hide(ActionOnInstanceLaunch::Hide, "hide", "show")]
    #[case::close(ActionOnInstanceLaunch::Close, "close", "create")]
    #[tokio::test]
    async fn should_bring_the_window_back_when_a_launched_game_ends(
        #[case] action: ActionOnInstanceLaunch,
        #[case] on_launch: &'static str,
        #[case] on_finish: &'static str,
    ) {
        let launched = windows_after(action, event(ProcessEventType::Launched, false)).await;
        let finished = windows_after(action, event(ProcessEventType::Finished, false)).await;

        assert_eq!(launched, [on_launch]);
        assert_eq!(finished, [on_finish]);
    }
}
