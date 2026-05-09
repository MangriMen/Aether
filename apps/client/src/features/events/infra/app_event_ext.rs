use async_trait::async_trait;
use tauri_specta::Event;

use crate::features::events::{AppEvent, ProgressEventDto};

#[async_trait]
pub trait AppEventExt {
    fn emit_to_tauri<R: tauri::Runtime>(&self, handle: &tauri::AppHandle<R>) -> tauri::Result<()>;
}

impl AppEventExt for AppEvent {
    fn emit_to_tauri<R: tauri::Runtime>(&self, handle: &tauri::AppHandle<R>) -> tauri::Result<()> {
        match self {
            AppEvent::Update(e) => ProgressEventDto::from(e.clone()).emit(handle),
        }
    }
}
