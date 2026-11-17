//! Giving a plugin a host on import.

use std::sync::{Arc, Mutex, atomic::Ordering};

use crate::features::instance::PluginId;
use crate::features::plugins::{AllowedHostRejection, HostAccess, PluginError, PluginSettings};

use super::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_the_host_without_a_restart_when_given_a_url() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let handle = harness.handle();

    let access = harness
        .grant_host(&settings)
        .execute(
            plugin_id(),
            "https://CDN.example.com/packs/pack.toml".into(),
            false,
        )
        .await
        .unwrap();

    assert_eq!(
        access,
        HostAccess::Granted {
            host: "cdn.example.com".into()
        }
    );
    assert_eq!(
        settings.hosts(),
        ["cdn.example.com"],
        "the host, not a pattern"
    );
    assert!(
        !handle.is_retired(),
        "hosts are read on every request, so the running plugin keeps going"
    );
    assert_eq!(harness.loader.loads.load(Ordering::SeqCst), 1);

    // Another file of the same host changes nothing.
    let again = harness
        .grant_host(&settings)
        .execute(
            plugin_id(),
            "http://cdn.example.com/other/pack.toml".into(),
            false,
        )
        .await
        .unwrap();
    assert_eq!(
        again,
        HostAccess::AlreadyAllowed {
            host: "cdn.example.com".into()
        }
    );
    assert_eq!(settings.hosts().len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_not_ask_for_a_host_when_the_manifest_or_a_pattern_admits_it() {
    let harness = Harness::enabled().await;
    harness
        .plugins
        .upsert_with(&plugin_id(), |plugin| {
            plugin.manifest.runtime.allowed_hosts = vec!["github.com".into()];
            Ok(())
        })
        .await
        .unwrap();
    let settings = Arc::new(MemoryPluginSettings(Mutex::new(Some(PluginSettings {
        allowed_hosts: vec!["*.example.com".into()],
        ..PluginSettings::default()
    }))));

    for (url, host) in [
        ("https://github.com/owner/pack/raw/pack.toml", "github.com"),
        ("https://a.example.com/pack.toml", "a.example.com"),
    ] {
        let access = harness
            .grant_host(&settings)
            .execute(plugin_id(), url.into(), true)
            .await
            .unwrap();
        assert_eq!(access, HostAccess::AlreadyAllowed { host: host.into() });
    }

    // The glob wants the dot, as Extism's does: the apex is a host of its own.
    let apex = harness
        .grant_host(&settings)
        .execute(plugin_id(), "https://example.com/pack.toml".into(), true)
        .await
        .unwrap();
    assert_eq!(
        apex,
        HostAccess::Needed {
            host: "example.com".into()
        }
    );
    assert_eq!(
        settings.hosts(),
        ["*.example.com"],
        "a dry run adds nothing"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_a_host_all_the_same_when_the_plugin_is_busy() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let handle = harness.handle();
    let call = harness.start_long_call(&handle).await;

    let access = harness
        .grant_host(&settings)
        .execute(plugin_id(), "https://example.com/pack.toml".into(), false)
        .await;

    assert!(
        matches!(access, Ok(HostAccess::Granted { .. })),
        "{access:?}"
    );
    assert_eq!(settings.hosts(), ["example.com"]);
    assert!(!handle.is_retired());
    harness.let_go();
    assert!(call.await.unwrap().is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_not_give_a_host_when_the_plugin_could_not_reach_it_anyway() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());

    for (url, expected) in [
        (
            r"D:\packs\pack.toml",
            AllowedHostRejection::UnsupportedScheme,
        ),
        (
            "ftp://example.com/pack.toml",
            AllowedHostRejection::UnsupportedScheme,
        ),
        (
            "http://localhost:8080/pack.toml",
            AllowedHostRejection::NotPublic,
        ),
        ("http://10.0.0.1/pack.toml", AllowedHostRejection::NotPublic),
        (
            "http://*.example.com/pack.toml",
            AllowedHostRejection::Wildcard,
        ),
    ] {
        for dry_run in [true, false] {
            let result = harness
                .grant_host(&settings)
                .execute(plugin_id(), url.into(), dry_run)
                .await;

            assert!(
                matches!(&result, Err(PluginError::InvalidAllowedHost { reason, .. }) if *reason == expected),
                "{url}: {result:?}"
            );
        }
    }
    assert_eq!(settings.hosts(), Vec::<String>::new());

    let unknown = harness
        .grant_host(&settings)
        .execute(
            PluginId::from_static("missing"),
            "https://example.com/".into(),
            false,
        )
        .await;
    assert!(
        matches!(unknown, Err(PluginError::NotFound { .. })),
        "{unknown:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_give_a_local_host_and_its_port_without_a_restart_when_granted() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());
    let handle = harness.handle();
    let use_case = harness.grant_local_host(&settings);
    let grant = |url: &str, dry_run| use_case.execute(plugin_id(), url.to_owned(), dry_run);

    let asked = grant("http://192.168.1.10:8080/", true).await.unwrap();
    assert_eq!(
        asked,
        HostAccess::Needed {
            host: "192.168.1.10:8080".into()
        }
    );
    assert!(settings.local_hosts().is_empty(), "a dry run adds nothing");

    let granted = grant("http://192.168.1.10:8080/", false).await.unwrap();
    assert_eq!(
        granted,
        HostAccess::Granted {
            host: "192.168.1.10:8080".into()
        }
    );
    // The port the URL names, else its scheme's: never every port of the host.
    grant("https://Localhost/", false).await.unwrap();
    assert_eq!(
        settings.local_hosts(),
        ["192.168.1.10:8080", "localhost:443"]
    );
    assert!(
        settings.hosts().is_empty(),
        "a local host is not an allowed_hosts entry"
    );
    assert!(!handle.is_retired());

    let again = grant("http://192.168.1.10:8080/other", false)
        .await
        .unwrap();
    assert_eq!(
        again,
        HostAccess::AlreadyAllowed {
            host: "192.168.1.10:8080".into()
        }
    );
    let other_port = grant("http://192.168.1.10:9090/", true).await.unwrap();
    assert!(
        matches!(other_port, HostAccess::Needed { .. }),
        "{other_port:?}"
    );
    assert_eq!(settings.local_hosts().len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_not_give_a_local_host_when_it_is_a_pattern_or_no_local_address() {
    let harness = Harness::enabled().await;
    let settings = Arc::new(MemoryPluginSettings::default());

    for (url, expected) in [
        (
            "ftp://192.168.1.10/",
            AllowedHostRejection::UnsupportedScheme,
        ),
        ("http://*.lan/", AllowedHostRejection::Wildcard),
        ("http://8.8.8.8/", AllowedHostRejection::NotLocal),
        ("http://0.0.0.0:8080/", AllowedHostRejection::NotLocal),
        ("http://224.0.0.1/", AllowedHostRejection::NotLocal),
    ] {
        let result = harness
            .grant_local_host(&settings)
            .execute(plugin_id(), url.into(), false)
            .await;

        assert!(
            matches!(&result, Err(PluginError::InvalidAllowedHost { reason, .. }) if *reason == expected),
            "{url}: {result:?}"
        );
    }
    assert!(settings.local_hosts().is_empty());

    let unknown = harness
        .grant_local_host(&settings)
        .execute(
            PluginId::from_static("missing"),
            "http://192.168.1.10/".into(),
            false,
        )
        .await;
    assert!(
        matches!(unknown, Err(PluginError::NotFound { .. })),
        "{unknown:?}"
    );
}
