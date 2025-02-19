use std::path::PathBuf;

use aether_core::{
    state::{Hooks, LauncherState, MemorySettings, WindowSize},
    utils::io::read_json_async,
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

    let settings = read_json_async::<aether_core::state::Settings>(settings_file)
        .await
        .unwrap_or_else(|_| {
            need_update_settings = true;

            aether_core::state::Settings {
                launcher_dir: user_data_dir.clone(),
                metadata_dir: user_data_dir.clone(),
                plugins_dir: user_data_dir.clone(),
                max_concurrent_downloads: 10,

                memory: MemorySettings { maximum: 1024 },
                game_resolution: WindowSize(960, 540),
                custom_env_vars: vec![],
                extra_launch_args: vec![],
                hooks: Hooks::default(),
            }
        });

    aether_core::state::LauncherState::init(&settings).await?;

    // let mut event_emitter = EventEmitter::new();

    // event_emitter.on("loading", move |payload: LoadingPayload| {
    //     let app = app.clone();
    //     app.emit("loading", payload).unwrap();
    // });

    // let event_emitter_arc = Arc::new(tokio::sync::Mutex::new(event_emitter));
    // // let event_emitter_arc = event_state.lock().await.event_emitter.clone();

    aether_core::state::EventState::init_with_app(app).await?;

    let state = LauncherState::get().await?;

    if need_update_settings {
        aether_core::state::Settings::update(&state, &settings).await?;
    }

    let mut plugin_manager = state.plugin_manager.write().await;

    plugin_manager.scan_plugins();
    plugin_manager.enable_plugin("packwiz".to_string()).await?;

    Ok(())
}
