use std::{
    io::{Cursor, Write},
    sync::Arc,
};

use async_trait::async_trait;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::features::plugins::{
    PluginProvider, PluginProviderError, PluginProviderFactory, PluginSourceType,
    PreviewPluginReleaseUseCase, ProviderPluginPreview, ProviderReleaseInfo, ProviderUpdateInfo,
    infra::ZipPluginExtractor,
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
    "api": {{ "version": "*", "features": [], "hostFunctions": [] }}
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
struct Releases;

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
        Ok(ProviderPluginPreview {
            owner: "owner".into(),
            repo: "repo".into(),
            releases: vec![
                release("v3", true),
                release("v2", false),
                release("v1", false),
            ],
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
    PreviewPluginReleaseUseCase::new(
        Arc::new(ZipPluginExtractor::default()),
        Arc::new(PluginProviderFactory::new(vec![Box::new(Releases)])),
    )
}

#[tokio::test]
async fn should_preview_the_picked_tag_from_its_archive_with_the_hash_and_size_of_it() {
    let preview = use_case()
        .execute(PluginSourceType::GitHub, "owner/repo", Some("v1".into()))
        .await
        .unwrap();

    let archive = Releases::archive("v1");
    assert_eq!(preview.tag, "v1");
    assert_eq!(preview.manifest.version, "0.1.0");
    assert_eq!(preview.archive_sha256, sha256_hex(&archive));
    assert_eq!(preview.size, archive.len() as u64);
    assert_eq!(preview.releases.len(), 3);
    assert_eq!(
        (preview.owner.as_str(), preview.repo.as_str()),
        ("owner", "repo")
    );
}

#[tokio::test]
async fn should_show_what_the_archive_asks_for_not_what_another_release_asks_for() {
    let preview = use_case()
        .execute(PluginSourceType::GitHub, "owner/repo", Some("v2".into()))
        .await
        .unwrap();

    assert_eq!(preview.manifest.allowed_hosts, ["example.com"]);
    assert_eq!(preview.manifest.host_functions, Some(Vec::new()));
    assert!(preview.capabilities.is_none());
}

#[tokio::test]
async fn should_preview_the_first_stable_release_when_no_tag_is_given() {
    let preview = use_case()
        .execute(PluginSourceType::GitHub, "owner/repo", None)
        .await
        .unwrap();

    assert_eq!(preview.tag, "v2");
}

#[tokio::test]
async fn should_fail_with_no_assets_when_the_tag_is_not_listed() {
    let result = use_case()
        .execute(PluginSourceType::GitHub, "owner/repo", Some("v9".into()))
        .await;

    assert!(matches!(
        result,
        Err(crate::features::plugins::PluginError::Provider(
            PluginProviderError::ProviderNoAssets { .. }
        ))
    ));
}
