use std::sync::Arc;

use aether_core::features::settings::LocationInfo;

use crate::{
    core::TauriWindowManager,
    features::{
        events::infra::TauriEventEmitter, settings::infra::SqliteAppSettingsStorage,
        update::infra::TauriUpdateService,
    },
};

pub type AppSettingsStorageState = Arc<SqliteAppSettingsStorage>;

pub type WindowManagerState<R> = Arc<TauriWindowManager<R>>;

pub type UpdateServiceState<R> = Arc<TauriUpdateService<R>>;

pub type EventEmitterState<R> = Arc<TauriEventEmitter<R>>;

pub type LocationInfoState = Arc<LocationInfo>;
