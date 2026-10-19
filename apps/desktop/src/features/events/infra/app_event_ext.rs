use async_trait::async_trait;
use tauri_specta::Event;

use crate::features::events::{
    AppEvent,
    infra::{ProgressEventDto, WithSeq},
};

#[async_trait]
pub trait AppEventExt {
    fn emit_to_tauri<R: tauri::Runtime>(
        &self,
        handle: &tauri::AppHandle<R>,
        seq: u32,
    ) -> tauri::Result<()>;
}

impl AppEventExt for AppEvent {
    fn emit_to_tauri<R: tauri::Runtime>(
        &self,
        handle: &tauri::AppHandle<R>,
        seq: u32,
    ) -> tauri::Result<()> {
        match self {
            AppEvent::Update(e) => ProgressEventDto::from(e.clone()).with_seq(seq).emit(handle),
        }
    }
}
