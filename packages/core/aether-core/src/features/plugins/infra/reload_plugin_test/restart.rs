//! Restarting a plugin: who gets the instance and what a busy plugin does.

use std::sync::{Arc, atomic::Ordering};

use crate::features::{
    instance::InstanceError,
    plugins::{EnablePluginUseCase, PluginError, PluginState, PluginStateReason},
};

use super::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_a_new_instance_and_leave_the_old_one_refusing_calls_when_restarted() {
    let harness = Harness::enabled().await;
    let old = harness.handle();

    harness.reload().execute(PLUGIN_ID.into()).await.unwrap();

    let new = harness.handle();
    assert!(old.is_retired());
    assert!(!new.is_retired());
    assert!(is_stopped(&noop(&old).await), "an old clone leads nowhere");
    assert!(noop(&new).await.is_ok());
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 2);
    assert_eq!(harness.loader.unloads.load(Ordering::SeqCst), 1);
    assert!(harness.is_enabled(), "a restart leaves the plugin enabled");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_not_restart_a_busy_plugin_and_let_its_call_finish_when_asked_to() {
    let harness = Harness::enabled().await;
    let handle = harness.handle();
    let call = harness.start_long_call(&handle).await;

    let refused = tokio::time::timeout(PATIENCE, harness.reload().execute(PLUGIN_ID.into()))
        .await
        .expect("the restart is refused, not waited for");

    assert!(
        matches!(refused, Err(PluginError::Busy { .. })),
        "{refused:?}"
    );
    assert!(matches!(harness.state(), PluginState::Loaded(_)));
    assert!(!handle.is_retired());

    harness.let_go();
    assert!(call.await.unwrap().is_ok(), "the call runs to its end");
    harness.reload().execute(PLUGIN_ID.into()).await.unwrap();
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_not_disable_a_busy_plugin_when_asked_to() {
    let harness = Harness::enabled().await;
    let handle = harness.handle();
    let call = harness.start_long_call(&handle).await;

    let refused = harness.disable().execute(PLUGIN_ID.into()).await;

    assert!(
        matches!(refused, Err(PluginError::Busy { .. })),
        "{refused:?}"
    );
    assert!(matches!(harness.state(), PluginState::Loaded(_)));
    assert!(harness.is_enabled());
    harness.let_go();
    assert!(call.await.unwrap().is_ok());
}

/// A host function of the plugin restarting it: refused as busy, rather than waiting for the
/// call it is part of.
#[test]
fn should_refuse_a_restart_when_it_comes_from_inside_the_plugin() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let harness = runtime.block_on(Harness::enabled());
    let reload = harness.reload();
    let handle = harness.handle();

    let inner = runtime
        .block_on(async {
            tokio::time::timeout(
                PATIENCE,
                handle.run("work", move |_| {
                    tokio::runtime::Handle::current().block_on(reload.execute(PLUGIN_ID.into()))
                }),
            )
            .await
        })
        .expect("the restart should not deadlock")
        .unwrap();

    assert!(matches!(inner, Err(PluginError::Busy { .. })), "{inner:?}");
    assert!(!handle.is_retired());
    runtime.shutdown_background();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_an_explicit_error_to_a_call_when_it_comes_during_a_restart() {
    let harness = Arc::new(Harness::enabled().await);
    let old = harness.handle();
    harness.loader.hold_loads.store(true, Ordering::SeqCst);

    let restart = tokio::spawn({
        let harness = Arc::clone(&harness);
        async move { harness.reload().execute(PLUGIN_ID.into()).await }
    });
    harness.loader.load_started.notified().await;

    assert!(matches!(harness.state(), PluginState::Loading));
    harness.sync_registries().await;
    let importer = harness
        .importer()
        .await
        .expect("the importer stays registered over the restart");
    assert!(matches!(
        importer.import("pack.toml").await,
        Err(InstanceError::PluginNotRunning { plugin_id }) if plugin_id == PLUGIN_ID
    ));
    assert!(is_stopped(&noop(&old).await));
    assert!(matches!(
        harness.reload().execute(PLUGIN_ID.into()).await,
        Err(PluginError::LoadingInProgress { .. })
    ));

    harness.loader.load_released.notify_one();
    restart.await.unwrap().unwrap();
    harness.sync_registries().await;
    let importer = harness.importer().await.expect("registered again");
    assert_eq!(
        importer.import("pack.toml").await.unwrap(),
        Vec::<String>::new()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_leave_the_plugin_failed_and_enabled_when_a_restart_fails_to_load() {
    let harness = Harness::enabled().await;
    let old = harness.handle();
    harness.loader.fail_next_load.store(true, Ordering::SeqCst);

    let result = harness.reload().execute(PLUGIN_ID.into()).await;

    assert!(
        matches!(result, Err(PluginError::LoadFailed { .. })),
        "{result:?}"
    );
    let PluginState::Failed(reason) = harness.state() else {
        panic!("the plugin should be failed");
    };
    assert_eq!(
        reason,
        PluginStateReason::LoadFailed {
            details: "the wasm does not compile".into(),
        }
    );
    assert!(harness.is_enabled());
    assert!(
        old.is_retired(),
        "the old instance does not live on with its old rights"
    );
    assert!(is_stopped(&noop(&old).await));
    harness.sync_registries().await;
    assert!(harness.importer().await.is_none());

    // Enabling it again is the way back.
    EnablePluginUseCase::new(
        Arc::clone(&harness.plugins),
        Arc::clone(&harness.loaders),
        Arc::new(NoPluginSettings),
        harness.settings.clone(),
    )
    .execute(PLUGIN_ID.into())
    .await
    .unwrap();
    assert!(noop(&harness.handle()).await.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_end_the_instance_before_each_restart_when_restarts_follow_one_another() {
    let harness = Harness::enabled().await;
    let first = harness.handle();

    harness.reload().execute(PLUGIN_ID.into()).await.unwrap();
    let second = harness.handle();
    harness.reload().execute(PLUGIN_ID.into()).await.unwrap();
    let third = harness.handle();

    assert!(first.is_retired() && second.is_retired() && !third.is_retired());
    assert!(noop(&third).await.is_ok());
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 3);
    assert_eq!(harness.loader.unloads.load(Ordering::SeqCst), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_not_restart_a_plugin_when_it_is_not_loaded() {
    let harness = Harness::enabled().await;
    harness.disable().execute(PLUGIN_ID.into()).await.unwrap();

    let result = harness.reload().execute(PLUGIN_ID.into()).await;

    assert!(
        matches!(result, Err(PluginError::AlreadyUnloaded { .. })),
        "{result:?}"
    );
    assert!(matches!(harness.state(), PluginState::NotLoaded));
}
