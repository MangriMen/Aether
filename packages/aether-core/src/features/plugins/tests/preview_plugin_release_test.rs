use std::{
    io::{Cursor, Write},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use async_trait::async_trait;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::features::plugins::{
    ListPluginReleasesUseCase, PluginProvider, PluginProviderError, PluginProviderFactory,
    PluginSourceType, PreviewPluginReleaseUseCase, ProviderPluginPreview, ProviderReleaseInfo,
    ProviderUpdateInfo, ReleaseArchiveCache, infra::ZipPluginExtractor,
};
use crate::shared::hash::infra::sha256_hex;

fn manifest(version: &str, hosts: &str) -> String {
    format!(
        r#"{{
    "metadata": {{
        "id": "test-plugin",
        "name": "Test Plugin",
        "version": "{version}",
        "description": null,
        "authors": [],
        "license": null
    }},
    "runtime": {{ "allowedHosts": [{hosts}], "allowedPaths": [] }},
    "load": {{ "type": "extism", "file": "plugin.wasm", "memoryLimit": null }},
    "api": {{ "version": "*", "hostFunctions": [] }}
}}"#
    )
}

fn plugin_zip(version: &str, hosts: &str) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in [
        ("manifest.json", manifest(version, hosts).into_bytes()),
        ("plugin.wasm", b"\0asm".to_vec()),
    ] {
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn release(tag: &str, is_prerelease: bool) -> ProviderReleaseInfo {
    ProviderReleaseInfo {
        tag_name: tag.into(),
        version: tag.trim_start_matches('v').into(),
        is_prerelease,
        published_at: "2026-01-01T00:00:00Z".into(),
        html_url: format!("https://example.com/{tag}"),
        download_url: format!("https://example.com/{tag}.zip"),
    }
}

/// Lists v3 (a prerelease), v2 and v1; each one's archive is a different build.
#[derive(Default)]
struct Releases {
    downloads: Arc<AtomicUsize>,
    /// How many times the provider was asked for the list of releases.
    listings: Arc<AtomicUsize>,
    /// A newer stable release whose archive is missing, listed first.
    assetless_newest: bool,
}

impl Releases {
    fn archive(tag: &str) -> Vec<u8> {
        match tag {
            "v3" => plugin_zip("0.3.0", ""),
            "v2" => plugin_zip("0.2.0", r#""example.com""#),
            _ => plugin_zip("0.1.0", ""),
        }
    }
}

#[async_trait]
impl PluginProvider for Releases {
    async fn fetch_preview(&self, _: &str) -> Result<ProviderPluginPreview, PluginProviderError> {
        self.listings.fetch_add(1, Ordering::SeqCst);
        let mut releases = vec![
            release("v3", true),
            release("v2", false),
            release("v1", false),
        ];
        if self.assetless_newest {
            releases.insert(
                0,
                ProviderReleaseInfo {
                    download_url: String::new(),
                    ..release("v4", false)
                },
            );
        }
        Ok(ProviderPluginPreview {
            owner: "owner".into(),
            repo: "repo".into(),
            releases,
        })
    }

    async fn fetch_latest_version(
        &self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<ProviderUpdateInfo, PluginProviderError> {
        unimplemented!("the preview does not look for updates")
    }

    async fn download_plugin(
        &self,
        url: &str,
        dest: &mut tokio::fs::File,
    ) -> Result<u64, PluginProviderError> {
        use tokio::io::AsyncWriteExt;

        self.downloads.fetch_add(1, Ordering::SeqCst);
        let tag = url
            .trim_start_matches("https://example.com/")
            .trim_end_matches(".zip");
        let archive = Self::archive(tag);
        dest.write_all(&archive).await.unwrap();
        Ok(archive.len() as u64)
    }

    fn source_type(&self) -> PluginSourceType {
        PluginSourceType::GitHub
    }

    fn display_name(&self) -> String {
        "Stub source".to_owned()
    }

    fn parse_identifier(&self, raw: &str) -> Result<String, PluginProviderError> {
        Ok(raw.to_owned())
    }
}

fn use_case() -> PreviewPluginReleaseUseCase {
    use_case_counting(Arc::default())
}

fn factory(provider: Releases) -> Arc<PluginProviderFactory> {
    Arc::new(PluginProviderFactory::new(vec![Box::new(provider)]))
}

/// A preview that counts the archives its provider is asked to download.
fn use_case_counting(downloads: Arc<AtomicUsize>) -> PreviewPluginReleaseUseCase {
    PreviewPluginReleaseUseCase::new(
        Arc::new(ZipPluginExtractor::default()),
        factory(Releases {
            downloads,
            ..Releases::default()
        }),
        Arc::new(ReleaseArchiveCache::new()),
    )
}

#[tokio::test]
async fn should_download_a_tag_once_when_it_is_previewed_again() {
    let downloads = Arc::new(AtomicUsize::new(0));
    let use_case = use_case_counting(downloads.clone());

    for tag in ["v1", "v2", "v1"] {
        use_case
            .execute(PluginSourceType::GitHub, "owner/repo", tag)
            .await
            .unwrap();
    }

    // v1 and v2, each once.
    assert_eq!(downloads.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn should_preview_the_picked_tag_from_its_archive_with_the_hash_and_size_of_it() {
    let preview = use_case()
        .execute(PluginSourceType::GitHub, "owner/repo", "v1")
        .await
        .unwrap();

    let archive = Releases::archive("v1");
    assert_eq!(preview.tag, "v1");
    assert_eq!(preview.manifest.version, "0.1.0");
    assert_eq!(preview.archive_sha256, sha256_hex(&archive));
}

#[tokio::test]
async fn should_show_what_the_archive_asks_for_not_what_another_release_asks_for() {
    let preview = use_case()
        .execute(PluginSourceType::GitHub, "owner/repo", "v2")
        .await
        .unwrap();

    assert_eq!(preview.manifest.allowed_hosts, ["example.com"]);
    assert_eq!(preview.manifest.host_functions, Some(Vec::new()));
    assert!(preview.capabilities.is_none());
}

/// A list use case and a preview sharing one cache, as the container builds them.
fn list_and_preview(
    provider: Releases,
) -> (ListPluginReleasesUseCase, PreviewPluginReleaseUseCase) {
    let factory = factory(provider);
    let cache = Arc::new(ReleaseArchiveCache::new());

    (
        ListPluginReleasesUseCase::new(factory.clone(), cache.clone()),
        PreviewPluginReleaseUseCase::new(Arc::new(ZipPluginExtractor::default()), factory, cache),
    )
}

#[tokio::test]
async fn should_list_the_releases_with_the_first_stable_one_as_default_without_downloading() {
    let downloads = Arc::new(AtomicUsize::new(0));
    let (list, _) = list_and_preview(Releases {
        downloads: downloads.clone(),
        ..Releases::default()
    });

    let listing = list
        .execute(PluginSourceType::GitHub, "owner/repo")
        .await
        .unwrap();

    assert_eq!(listing.default_tag, "v2");
    assert_eq!(listing.releases.len(), 3);
    assert_eq!(
        (listing.owner.as_str(), listing.repo.as_str()),
        ("owner", "repo")
    );
    assert_eq!(downloads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn should_leave_out_a_release_without_an_archive_when_listing() {
    let (list, preview) = list_and_preview(Releases {
        assetless_newest: true,
        ..Releases::default()
    });

    let listing = list
        .execute(PluginSourceType::GitHub, "owner/repo")
        .await
        .unwrap();

    // v4 is the newest stable release but has nothing to install, so the default is v2.
    assert_eq!(listing.default_tag, "v2");
    assert!(listing.releases.iter().all(|r| r.tag_name != "v4"));
    assert!(
        preview
            .execute(PluginSourceType::GitHub, "owner/repo", "v4")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn should_ask_the_provider_once_when_the_list_is_followed_by_a_preview() {
    let listings = Arc::new(AtomicUsize::new(0));
    let (list, preview) = list_and_preview(Releases {
        listings: listings.clone(),
        ..Releases::default()
    });

    list.execute(PluginSourceType::GitHub, "owner/repo")
        .await
        .unwrap();
    preview
        .execute(PluginSourceType::GitHub, "owner/repo", "v1")
        .await
        .unwrap();
    preview
        .execute(PluginSourceType::GitHub, "owner/repo", "v2")
        .await
        .unwrap();

    assert_eq!(listings.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn should_fail_with_no_assets_when_the_tag_is_not_listed() {
    let result = use_case()
        .execute(PluginSourceType::GitHub, "owner/repo", "v9")
        .await;

    assert!(matches!(
        result,
        Err(crate::features::plugins::PluginError::Provider(
            PluginProviderError::ProviderNoAssets { .. }
        ))
    ));
}
