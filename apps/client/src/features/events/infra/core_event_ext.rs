use aether_core::features::events::Event as CoreEvent;
use async_trait::async_trait;
use tauri_specta::Event;

use crate::features::{
    events::{ProgressEventDto, WarningEventDto},
    instance::InstanceEventDto,
    plugins::PluginEventDto,
    process::ProcessEventDto,
};

#[async_trait]
pub trait CoreEventExt {
    fn emit_to_tauri<R: tauri::Runtime>(&self, handle: &tauri::AppHandle<R>) -> tauri::Result<()>;
}

impl CoreEventExt for CoreEvent {
    fn emit_to_tauri<R: tauri::Runtime>(&self, handle: &tauri::AppHandle<R>) -> tauri::Result<()> {
        match self {
            CoreEvent::Instance(e) => InstanceEventDto::from(e.clone()).emit(handle),
            CoreEvent::Process(e) => ProcessEventDto::from(e.clone()).emit(handle),
            CoreEvent::Progress(e) => ProgressEventDto::from(e.clone()).emit(handle),
            CoreEvent::Plugin(e) => PluginEventDto::from(e.clone()).emit(handle),
            CoreEvent::Warning(e) => WarningEventDto::from(e.clone()).emit(handle),
        }
    }
}
