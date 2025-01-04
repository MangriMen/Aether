use std::path::PathBuf;

use aether_core::{
    event::LoadingBar,
    state::{Hooks, Instance, LauncherState, MemorySettings, MinecraftProcessMetadata, WindowSize},
    utils::io::read_json_async,
};
use daedalus::minecraft;
use dashmap::DashMap;
use tauri::AppHandle;
use uuid::Uuid;

use crate::{models::minecraft::InstanceCreateDto, utils::result::AetherLauncherResult};

#[tauri::command]
pub async fn initialize_state(app: AppHandle) -> AetherLauncherResult<()> {
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

    aether_core::event::EventState::init_with_app(app).await?;

    let state = LauncherState::get().await?;

    state.load_plugin("packwiz".to_string()).await?;

    aether_core::state::Settings::update(&state, &settings).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_minecraft_version_manifest() -> AetherLauncherResult<minecraft::VersionManifest> {
    let state = LauncherState::get().await?;
    let manifest = aether_core::launcher::download_version_manifest(&state, false).await?;

    Ok(manifest)
}

#[tauri::command]
pub async fn create_minecraft_instance(
    instance_create_dto: InstanceCreateDto,
) -> AetherLauncherResult<String> {
    Ok(aether_core::api::instance::instance_create(
        instance_create_dto.name,
        instance_create_dto.game_version,
        instance_create_dto.mod_loader,
        instance_create_dto.loader_version,
        instance_create_dto.icon_path,
        instance_create_dto.skip_install_profile,
    )
    .await?)
}

#[tauri::command]
pub async fn get_minecraft_instances() -> AetherLauncherResult<(Vec<Instance>, Vec<String>)> {
    let res = aether_core::api::instance::get_all().await?;
    Ok((res.0, res.1.iter().map(|err| err.to_string()).collect()))
}

#[tauri::command]
pub async fn launch_minecraft_instance(
    id: String,
) -> AetherLauncherResult<MinecraftProcessMetadata> {
    Ok(aether_core::api::instance::run(&id).await?)
}

#[tauri::command]
pub async fn stop_minecraft_instance(uuid: Uuid) -> AetherLauncherResult<()> {
    Ok(aether_core::api::process::kill(uuid).await?)
}

#[tauri::command]
pub async fn remove_minecraft_instance(id: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::remove(&id).await?)
}

#[tauri::command]
pub async fn get_progress_bars() -> AetherLauncherResult<DashMap<Uuid, LoadingBar>> {
    Ok(aether_core::event::EventState::list_progress_bars().await?)
}

#[tauri::command]
pub async fn get_running_minecraft_instances() -> AetherLauncherResult<Vec<MinecraftProcessMetadata>>
{
    Ok(aether_core::api::process::get_all().await?)
}

#[tauri::command]
pub async fn get_minecraft_instance_process(
    id: String,
) -> AetherLauncherResult<Vec<MinecraftProcessMetadata>> {
    Ok(aether_core::api::process::get_by_instance_id(&id).await?)
}
