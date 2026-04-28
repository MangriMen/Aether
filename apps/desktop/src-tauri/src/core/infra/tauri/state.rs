use std::sync::Arc;

use crate::{
    core::TauriWindowManager,
    features::{
        events::TauriEventEmitter, settings::FsAppSettingsStorage, update::TauriUpdateService,
    },
};

pub type AppSettingsStorageState = Arc<FsAppSettingsStorage>;

pub type WindowManagerState<R> = Arc<TauriWindowManager<R>>;

pub type UpdateServiceState<R> = Arc<TauriUpdateService<R>>;

pub type EventEmitterState<R> = Arc<TauriEventEmitter<R>>;
