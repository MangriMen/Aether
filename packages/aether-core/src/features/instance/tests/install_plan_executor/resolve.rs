//! Resolving refs and choosing the provider of a platform.

use crate::{
    features::{
        instance::{ContentSource, InstallPlan, InstallTarget, InstanceError, NewInstance},
        minecraft::ModLoader,
    },
    shared::hash::FileHash,
};

use super::common::*;

/// A ref becomes the download its platform's provider names, asked
/// with the instance's game version and loader, and lands where the plan puts it.
#[tokio::test]
async fn should_resolve_a_provider_ref_to_a_direct_download_when_installing() {
    let fixture = Fixture::new().await;
    let modrinth = MockProvider::builtin_modrinth();
    fixture.register(&modrinth).await;

    let instance_id = fixture
        .execute(
            existing(vec![provider_ref(
                "modrinth",
                "AANobbMI",
                "mods/sodium.jar",
            )]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(instance_id.as_str(), INSTANCE_ID);
    // One `resolve_file` per ref: its answer is a URL, so nothing is left to resolve.
    assert_eq!(
        modrinth.calls(),
        [(
            "AANobbMI".into(),
            None,
            "1.21.1".into(),
            Some("fabric".into())
        )]
    );
    // The URL is the resolving provider's word, not the word of the plan's provider.
    assert_eq!(
        fixture.downloader.downloads(),
        [Download {
            origin: "aether#modrinth".into(),
            url: modrinth.url_for("AANobbMI"),
            hash: Some(FileHash::Sha1(SHA1_ABC.into())),
        }]
    );
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "mods/sodium.jar"),
        Some(modrinth.url_for("AANobbMI"))
    );
    assert!(fixture.staging_is_clean());
}

#[tokio::test]
async fn should_report_progress_up_to_complete_when_the_plan_finishes() {
    let fixture = Fixture::new().await;

    fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/pack/a.jar", "mods/a.jar"),
                direct("https://cdn.example/pack/b.jar", "mods/b.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    let (total, steps) = fixture.progress.bar();
    let messages: Vec<_> = steps.iter().filter_map(|(_, m)| m.as_deref()).collect();

    // Download, hash and install per file, then the last step.
    assert!((total - 7.0).abs() < f64::EPSILON);
    assert!((steps.iter().map(|(inc, _)| inc).sum::<f64>() - total).abs() < f64::EPSILON);
    assert_eq!(messages.iter().filter(|m| **m == "Downloading").count(), 2);
    assert_eq!(messages.iter().filter(|m| **m == "Verifying").count(), 2);
    assert_eq!(messages.iter().filter(|m| **m == "Installing").count(), 2);
    assert_eq!(messages.last(), Some(&"Complete"));
}

#[tokio::test]
async fn should_caption_the_bar_with_the_instance_name_when_the_plan_runs() {
    let fixture = Fixture::new().await;
    let new_instance = NewInstance {
        name: "My Pack".into(),
        game_version: "1.20.1".into(),
        mod_loader: ModLoader::Vanilla,
        loader_version: None,
        icon_path: None,
        skip_install_instance: Some(true),
        pack_info: None,
    };

    fixture
        .execute(
            InstallPlan {
                target: InstallTarget::New(new_instance),
                files: vec![direct("https://cdn.example/pack/a.jar", "mods/a.jar")],
                overrides: Vec::new(),
            },
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.progress.caption.lock().unwrap().as_deref(),
        Some("My Pack")
    );
}

#[tokio::test]
async fn should_move_the_bar_inside_a_file_when_bytes_are_received() {
    let fixture = Fixture::new().await;
    let url = "https://cdn.example/chunks/big.jar";
    let mut big = direct(url, "mods/big.jar");
    big.size = Some(url.len() as u64);

    fixture
        .execute(existing(vec![big]), &plugin("packwiz"))
        .await
        .unwrap();

    let (total, steps) = fixture.progress.bar();
    let downloading: Vec<f64> = steps
        .iter()
        .filter(|(_, message)| message.as_deref() == Some("Downloading"))
        .map(|(increment, _)| *increment)
        .collect();

    assert!(downloading.len() >= 4, "{downloading:?}");
    assert!(downloading.iter().all(|increment| *increment > 0.0));
    assert!((downloading.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    assert!((steps.iter().map(|(inc, _)| inc).sum::<f64>() - total).abs() < 1e-9);
}

#[tokio::test]
async fn should_report_the_bytes_of_the_pack_when_every_size_is_known() {
    let fixture = Fixture::new().await;
    let big_url = "https://cdn.example/chunks/big.jar";
    let small_url = "https://cdn.example/pack/small.jar";
    let mut big = direct(big_url, "mods/big.jar");
    big.size = Some(big_url.len() as u64);
    let mut small = direct(small_url, "mods/small.jar");
    small.size = Some(small_url.len() as u64);

    fixture
        .execute(existing(vec![big, small]), &plugin("packwiz"))
        .await
        .unwrap();

    let expected = (big_url.len() + small_url.len()) as u64;
    let bytes = fixture.progress.bytes.lock().unwrap().clone();
    assert_eq!(
        *fixture.progress.bytes_total.lock().unwrap(),
        Some(expected)
    );
    // Four pieces of the big file as they arrive; the small one, which the downloader does not
    // report, by the size it returns.
    assert_eq!(bytes.len(), 5, "{bytes:?}");
    assert_eq!(bytes.iter().sum::<u64>(), expected);
}

#[tokio::test]
async fn should_leave_the_pack_total_unknown_when_a_size_is_unknown() {
    let fixture = Fixture::new().await;
    let known_url = "https://cdn.example/chunks/known.jar";
    let unknown_url = "https://cdn.example/pack/unknown.jar";
    let mut known = direct(known_url, "mods/known.jar");
    known.size = Some(known_url.len() as u64);

    fixture
        .execute(
            existing(vec![known, direct(unknown_url, "mods/unknown.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(*fixture.progress.bytes_total.lock().unwrap(), None);
    assert_eq!(
        fixture.progress.bytes.lock().unwrap().iter().sum::<u64>(),
        (known_url.len() + unknown_url.len()) as u64
    );
}

#[tokio::test]
async fn should_not_move_the_bar_past_a_file_when_its_size_was_understated() {
    let fixture = Fixture::new().await;
    let url = "https://cdn.example/chunks/liar.jar";
    let mut liar = direct(url, "mods/liar.jar");
    liar.size = Some(5);

    fixture
        .execute(existing(vec![liar]), &plugin("packwiz"))
        .await
        .unwrap();

    let (total, steps) = fixture.progress.bar();
    let downloading: f64 = steps
        .iter()
        .filter(|(_, message)| message.as_deref() == Some("Downloading"))
        .map(|(increment, _)| *increment)
        .sum();

    assert!((downloading - 1.0).abs() < 1e-9);
    assert!((steps.iter().map(|(inc, _)| inc).sum::<f64>() - total).abs() < 1e-9);
}

#[tokio::test]
async fn should_download_a_direct_url_under_the_origin_policy_when_the_plan_has_an_origin() {
    let fixture = Fixture::new().await;

    fixture
        .execute(
            existing(vec![direct("https://cdn.example/pack/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(
        fixture.downloader.downloads(),
        [Download {
            origin: "packwiz#provider".into(),
            url: "https://cdn.example/pack/a.jar".into(),
            hash: None,
        }]
    );
}

#[tokio::test]
async fn should_pass_the_ref_version_through_unchanged_when_resolving() {
    let fixture = Fixture::new().await;
    let modrinth = MockProvider::builtin_modrinth();
    fixture.register(&modrinth).await;

    let mut entry = provider_ref("modrinth", "AANobbMI", "mods/sodium.jar");
    entry.source = ContentSource::ProviderRef {
        platform: "modrinth".into(),
        content_id: "AANobbMI".into(),
        version: Some("IZskON6d".into()),
    };
    fixture
        .execute(existing(vec![entry]), &plugin("packwiz"))
        .await
        .unwrap();

    assert_eq!(modrinth.calls()[0].1.as_deref(), Some("IZskON6d"));
}

#[tokio::test]
async fn should_refuse_a_ref_when_no_provider_serves_its_platform() {
    let fixture = Fixture::new().await;
    // Serves no platform at all, so it can never answer a ref.
    fixture
        .register(&MockProvider::new("packwiz", "packwiz", None))
        .await;

    let err = fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/a.jar", "mods/a.jar"),
                provider_ref("curseforge", "238222", "mods/jei.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::PlatformProviderNotFound { platform } if platform == "curseforge"),
        "{err:?}"
    );
    assert!(fixture.downloader.downloads().is_empty());
    assert!(fixture.instance_files(INSTANCE_ID).is_empty());
}

#[tokio::test]
async fn should_fail_the_plan_when_a_provider_cannot_resolve_a_ref() {
    let fixture = Fixture::new().await;
    fixture.register(&MockProvider::builtin_modrinth()).await;

    let err = fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/a.jar", "mods/a.jar"),
                provider_ref("modrinth", "unsupported", "mods/b.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::UnsupportedOperation { .. }),
        "{err:?}"
    );
    assert!(fixture.downloader.downloads().is_empty());
}

// ─── Choosing the provider of a platform ───────────────────────

#[tokio::test]
async fn should_use_the_only_provider_when_a_platform_has_one() {
    let fixture = Fixture::new().await;
    let curseforge = MockProvider::new("curseforge-plugin", "curseforge", Some("curseforge"));
    fixture.register(&curseforge).await;

    fixture
        .execute(
            existing(vec![provider_ref("curseforge", "238222", "mods/jei.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(curseforge.calls().len(), 1);
    assert_eq!(
        fixture.downloader.downloads()[0].origin,
        "curseforge-plugin#curseforge"
    );
}

#[tokio::test]
async fn should_prefer_the_builtin_provider_when_nothing_is_chosen() {
    let fixture = Fixture::new().await;
    let other = MockProvider::new("modrinth-plus", "modrinth", Some("modrinth"));
    let builtin = MockProvider::builtin_modrinth();
    fixture.register(&other).await;
    fixture.register(&builtin).await;

    fixture
        .execute(
            existing(vec![provider_ref(
                "modrinth",
                "AANobbMI",
                "mods/sodium.jar",
            )]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(builtin.calls().len(), 1);
    assert!(other.calls().is_empty());
}

#[tokio::test]
async fn should_ask_for_a_choice_when_several_plugin_providers_serve_a_platform() {
    let fixture = Fixture::new().await;
    let first = MockProvider::new("cf-one", "curseforge", Some("curseforge"));
    let second = MockProvider::new("cf-two", "curseforge", Some("curseforge"));
    fixture.register(&first).await;
    fixture.register(&second).await;

    let err = fixture
        .execute(
            existing(vec![provider_ref("curseforge", "238222", "mods/jei.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::PlatformProviderNotChosen { platform } if platform == "curseforge"),
        "{err:?}"
    );
    assert!(first.calls().is_empty() && second.calls().is_empty());
}

#[tokio::test]
async fn should_use_the_chosen_provider_when_a_builtin_one_exists_too() {
    let mut fixture = Fixture::new().await;
    let chosen = MockProvider::new("modrinth-plus", "modrinth", Some("modrinth"));
    let builtin = MockProvider::builtin_modrinth();
    fixture.register(&chosen).await;
    fixture.register(&builtin).await;
    fixture.selected = Some(chosen.id.clone());

    fixture
        .execute(
            existing(vec![provider_ref(
                "modrinth",
                "AANobbMI",
                "mods/sodium.jar",
            )]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(chosen.calls().len(), 1);
    assert!(builtin.calls().is_empty());
}

/// A chosen provider that went away is not quietly replaced, not even by the built-in one.
#[tokio::test]
async fn should_not_switch_provider_when_the_chosen_one_is_unavailable() {
    let mut fixture = Fixture::new().await;
    let builtin = MockProvider::builtin_modrinth();
    fixture.register(&builtin).await;
    fixture.selected = Some(plugin("uninstalled"));

    let err = fixture
        .execute(
            existing(vec![provider_ref(
                "modrinth",
                "AANobbMI",
                "mods/sodium.jar",
            )]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, InstanceError::PlatformProviderNotChosen { .. }),
        "{err:?}"
    );
    assert!(builtin.calls().is_empty());
}
