use aether_core::features::events::Event as CoreEvent;
use async_trait::async_trait;
use tauri_specta::Event;

use crate::features::{
    events::infra::{CacheEventDto, ProgressEventDto, WarningEventDto, WithSeq},
    instance::infra::InstanceEventDto,
    plugins::infra::PluginEventDto,
    process::infra::ProcessEventDto,
};

#[async_trait]
pub trait CoreEventExt {
    fn emit_to_tauri<R: tauri::Runtime>(
        &self,
        handle: &tauri::AppHandle<R>,
        seq: u32,
    ) -> tauri::Result<()>;
}

impl CoreEventExt for CoreEvent {
    fn emit_to_tauri<R: tauri::Runtime>(
        &self,
        handle: &tauri::AppHandle<R>,
        seq: u32,
    ) -> tauri::Result<()> {
        match self {
            CoreEvent::Instance(e) => InstanceEventDto::from(e.clone()).with_seq(seq).emit(handle),
            CoreEvent::Process(e) => ProcessEventDto::from(e.clone()).with_seq(seq).emit(handle),
            CoreEvent::Progress(e) => ProgressEventDto::from(e.clone()).with_seq(seq).emit(handle),
            CoreEvent::Plugin(e) => PluginEventDto::from(e.clone()).with_seq(seq).emit(handle),
            CoreEvent::Warning(e) => WarningEventDto::from(e.clone()).with_seq(seq).emit(handle),
            CoreEvent::Cache(e) => CacheEventDto::from(e.clone()).with_seq(seq).emit(handle),
        }
    }
}
