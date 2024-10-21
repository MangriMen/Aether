use aether_core::{
    event::LoadingBar,
    state::{Instance, LauncherState, MinecraftProcessMetadata},
};
use daedalus::minecraft;
use dashmap::DashMap;
use tauri::AppHandle;
use uuid::Uuid;

use crate::{
    models::minecraft::InstanceCreateDto,
    utils::{self, result::AetherLauncherResult},
};

#[tauri::command]
pub async fn initialize_state(app: AppHandle) -> AetherLauncherResult<()> {
    let user_data_dir = Some(utils::tauri::get_app_dir(&app).to_str().unwrap().to_owned());

    let settings = aether_core::state::Settings {
        launcher_dir: user_data_dir.clone(),
        metadata_dir: user_data_dir.clone(),
        max_concurrent_downloads: 4,
    };

    aether_core::state::LauncherState::init(&settings).await?;

    // let mut event_emitter = EventEmitter::new();

    // event_emitter.on("loading", move |payload: LoadingPayload| {
    //     let app = app.clone();
    //     app.emit("loading", payload).unwrap();
    // });

    // let event_emitter_arc = Arc::new(tokio::sync::Mutex::new(event_emitter));
    // // let event_emitter_arc = event_state.lock().await.event_emitter.clone();

    aether_core::event::EventState::init_with_app(app).await?;

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
        instance_create_dto.linked_data,
        instance_create_dto.skip_install_profile,
    )
    .await?)
}

#[tauri::command]
pub async fn get_minecraft_instances() -> AetherLauncherResult<Vec<Instance>> {
    Ok(aether_core::api::instance::get_instances().await?)
}

#[tauri::command]
pub async fn launch_minecraft_instance(
    name_id: String,
) -> AetherLauncherResult<MinecraftProcessMetadata> {
    Ok(aether_core::api::instance::run(&name_id).await?)
}

#[tauri::command]
pub async fn remove_minecraft_instance(name_id: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::instance::remove(&name_id).await?)
}

#[tauri::command]
pub async fn get_progress_bars() -> AetherLauncherResult<DashMap<Uuid, LoadingBar>> {
    Ok(aether_core::event::EventState::list_progress_bars().await?)
}
