//! Enabling and disabling a plugin at the same time.

use std::sync::{Arc, atomic::Ordering};

use crate::features::plugins::{
    EnablePluginUseCase, PluginError, PluginLoaderRegistry, PluginSettingsStorage, PluginState,
    PluginStateReason,
};

use super::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_build_one_instance_when_the_plugin_is_enabled_twice_at_once() {
    let harness = Arc::new(Harness::enabled().await);
    harness.disable().execute(PLUGIN_ID.into()).await.unwrap();
    harness.loader.hold_loads.store(true, Ordering::SeqCst);

    let first = tokio::spawn({
        let harness = Arc::clone(&harness);
        async move { harness.enable().execute(PLUGIN_ID.into()).await }
    });
    harness.loader.load_started.notified().await;
    let second = harness.enable().execute(PLUGIN_ID.into()).await;
    let forced = harness.force_enable().execute(PLUGIN_ID.into()).await;

    assert!(
        matches!(second, Err(PluginError::LoadingInProgress { .. })),
        "{second:?}"
    );
    assert!(
        matches!(forced, Err(PluginError::LoadingInProgress { .. })),
        "{forced:?}"
    );
    harness.loader.load_released.notify_one();
    first.await.unwrap().unwrap();
    assert_eq!(
        harness.loader.loads.load(Ordering::SeqCst),
        2,
        "the first enable of the harness and one of these"
    );
    assert!(noop(&harness.handle()).await.is_ok());
}

/// Each enable reads the settings, slowly, after it has looked at the state: both look before
/// either loads, so the state has to be claimed by the look itself.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_build_one_instance_when_two_enables_check_the_state_at_once() {
    let harness = Harness::enabled().await;
    harness.disable().execute(PLUGIN_ID.into()).await.unwrap();
    let settings = Arc::new(SlowPluginSettings::default());
    let enable = || {
        EnablePluginUseCase::new(
            Arc::clone(&harness.plugins),
            Arc::clone(&harness.loaders),
            Arc::clone(&settings) as Arc<dyn PluginSettingsStorage>,
            harness.settings.clone(),
        )
    };
    let (first, second) = (enable(), enable());

    let (first, second) = tokio::join!(
        first.execute(PLUGIN_ID.into()),
        second.execute(PLUGIN_ID.into())
    );

    let refused = [&first, &second]
        .into_iter()
        .filter(|result| matches!(result, Err(PluginError::LoadingInProgress { .. })))
        .count();
    assert_eq!(refused, 1, "{first:?} {second:?}");
    assert!(first.is_ok() || second.is_ok(), "{first:?} {second:?}");
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_refuse_a_disable_when_the_plugin_is_being_restarted() {
    let harness = Arc::new(Harness::enabled().await);
    harness.loader.hold_loads.store(true, Ordering::SeqCst);
    let restart = tokio::spawn({
        let harness = Arc::clone(&harness);
        async move { harness.reload().execute(PLUGIN_ID.into()).await }
    });
    harness.loader.load_started.notified().await;

    let refused = harness.disable().execute(PLUGIN_ID.into()).await;

    assert!(
        matches!(refused, Err(PluginError::LoadingInProgress { .. })),
        "{refused:?}"
    );
    harness.loader.load_released.notify_one();
    restart.await.unwrap().unwrap();
    assert!(harness.is_enabled());

    // Once the restart is over, the disable goes through and nothing brings the plugin back.
    harness.disable().execute(PLUGIN_ID.into()).await.unwrap();
    assert!(!harness.is_enabled());
    assert!(matches!(harness.state(), PluginState::NotLoaded));
}

/// A restart takes the instance before loading the next one: a disable then must not report
/// success while the restart goes on to start the plugin again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_refuse_a_disable_when_a_restart_has_taken_the_instance() {
    let harness = Harness::enabled().await;
    harness
        .plugins
        .upsert_with(PLUGIN_ID, |plugin| {
            plugin.state = PluginState::Unloading;
            Ok(())
        })
        .await
        .unwrap();

    let refused = harness.disable().execute(PLUGIN_ID.into()).await;

    assert!(
        matches!(refused, Err(PluginError::UnloadingInProgress { .. })),
        "{refused:?}"
    );
    assert!(harness.is_enabled());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_leave_the_plugin_as_it_was_when_enabling_fails_before_the_load() {
    let harness = Harness::enabled().await;
    harness.disable().execute(PLUGIN_ID.into()).await.unwrap();
    harness
        .plugins
        .upsert_with(PLUGIN_ID, |plugin| {
            plugin.state = PluginState::Failed(PluginStateReason::LoadFailed {
                details: "the wasm does not compile".into(),
            });
            Ok(())
        })
        .await
        .unwrap();
    let no_loaders = EnablePluginUseCase::new(
        Arc::clone(&harness.plugins),
        Arc::new(PluginLoaderRegistry::default()),
        Arc::new(NoPluginSettings),
        harness.settings.clone(),
    );

    let result = no_loaders.execute(PLUGIN_ID.into()).await;

    assert!(
        matches!(result, Err(PluginError::LoaderNotFound { .. })),
        "{result:?}"
    );
    assert!(
        matches!(harness.state(), PluginState::Failed(_)),
        "not left `Loading`, which would refuse every later enable"
    );
}
