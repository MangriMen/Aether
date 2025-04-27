use std::{collections::HashSet, path::PathBuf};

use aether_core::{
    core::{domain::ServiceLocator, LauncherState},
    features::{
        events::EventState,
        settings::{Hooks, MemorySettings, Settings, WindowSize},
    },
    shared::read_json_async,
};
use tauri::AppHandle;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn initialize_state(app: AppHandle) -> AetherLauncherResult<()> {
    if LauncherState::initialized().await {
        return Ok(());
    }

    let user_data_dir = Some(
        crate::utils::tauri::get_app_dir(&app)
            .to_str()
            .unwrap()
            .to_owned(),
    );

    let settings_file = PathBuf::from(&user_data_dir.clone().unwrap()).join("settings.json");

    let mut need_update_settings = false;

    let settings = read_json_async::<Settings>(settings_file)
        .await
        .unwrap_or_else(|_| {
            need_update_settings = true;

            Settings {
                launcher_dir: user_data_dir.clone(),
                metadata_dir: user_data_dir.clone(),
                max_concurrent_downloads: 10,

                memory: MemorySettings { maximum: 2048 },
                game_resolution: WindowSize(960, 540),
                custom_env_vars: vec![],
                extra_launch_args: vec![],
                hooks: Hooks::default(),

                enabled_plugins: HashSet::default(),
            }
        });

    LauncherState::init(&settings).await?;

    // let mut event_emitter = EventEmitter::new();

    // event_emitter.on("loading", move |payload: LoadingPayload| {
    //     let app = app.clone();
    //     app.emit("loading", payload).unwrap();
    // });

    // let event_emitter_arc = Arc::new(tokio::sync::Mutex::new(event_emitter));
    // // let event_emitter_arc = event_state.lock().await.event_emitter.clone();

    EventState::init_with_app(app).await?;

    let state = LauncherState::get().await?;

    if need_update_settings {
        if let Err(e) = aether_core::api::settings::upsert(&settings).await {
            log::error!("Failed to update settings: {}", e);
        }
    }

    tokio::fs::create_dir_all(&state.locations.plugins_dir())
        .await
        .unwrap();

    let service_locator = ServiceLocator::get().await?;
    let mut plugin_service = service_locator.plugin_service.write().await;

    if let Err(e) = plugin_service.scan_plugins().await {
        log::error!("Failed to scan plugins: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub async fn load_enabled_plugins() -> AetherLauncherResult<()> {
    let service_locator = ServiceLocator::get().await?;
    let mut plugin_service = service_locator.plugin_service.write().await;

    let settings = aether_core::api::settings::get().await?;

    for plugin in settings.enabled_plugins.iter() {
        if let Err(e) = plugin_service.enable(plugin).await {
            log::error!("Failed to load plugin {}: {}", plugin, e);
        }
    }

    Ok(())
}
