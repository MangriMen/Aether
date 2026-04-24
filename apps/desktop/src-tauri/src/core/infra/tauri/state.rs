use std::sync::Arc;

use crate::features::{
    events::TauriEventEmitter,
    settings::{FsAppSettingsStorage, TauriWindowManager},
    update::TauriUpdateService,
};

pub type AppSettingsStorageState = Arc<FsAppSettingsStorage>;

pub type WindowManagerState<R> = Arc<TauriWindowManager<R>>;

pub type UpdateServiceState<R> = Arc<TauriUpdateService<R>>;

pub type EventEmitterState<R> = Arc<TauriEventEmitter<R>>;
