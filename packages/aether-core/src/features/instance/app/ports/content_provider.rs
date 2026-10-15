use std::collections::HashMap;

use async_trait::async_trait;

use crate::features::instance::app::dtos::{
    ContentCompatibilityCheckParams, ContentCompatibilityResult,
};
use crate::features::instance::domain::{
    ContentItem, ContentMetadata, ContentProviderCapabilityMetadata, ContentSearchParams,
    ContentSearchResult, ContentVersion, InstallPlan, Instance, InstanceError, PackMetadata,
    PackSource, ProviderId, ResolveInstallPlanParams, ResolvedFile,
};

#[async_trait]
pub trait ContentProvider: Send + Sync {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata;

    /// The id this provider is registered under (`aether#modrinth`, `<plugin>#<capability>`).
    /// [`ContentProviderCapabilityMetadata`] holds only the capability half of it.
    fn provider_id(&self) -> ProviderId;

    async fn search(
        &self,
        search_content: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError>;

    async fn get_content(&self, content_id: String) -> Result<ContentItem, InstanceError>;

    async fn list_versions(&self, content_id: String)
    -> Result<Vec<ContentVersion>, InstanceError>;

    /// Which file a [`ContentSource::ProviderRef`] of this provider's platform stands for.
    ///
    /// `version: None` asks for the newest file that runs on `game_version` with `loader`
    /// (`fabric`, `neoforge`; `None` — vanilla); with a `version`, the provider returns that file
    /// and need not check it against them. No platform parameter: the core picked this provider
    /// by its [`ContentProviderCapabilityMetadata::platform`].
    ///
    /// [`ContentSource::ProviderRef`]: crate::features::instance::ContentSource::ProviderRef
    async fn resolve_file(
        &self,
        content_id: &str,
        version: Option<&str>,
        game_version: &str,
        loader: Option<&str>,
    ) -> Result<ResolvedFile, InstanceError> {
        let _ = (content_id, version, game_version, loader);
        Err(self.unsupported("resolve_file"))
    }

    /// The full plan for installing or updating content, one mod and a whole modpack alike.
    /// See [`ResolveTarget::Existing`](crate::features::instance::ResolveTarget::Existing) for
    /// when the plan may name a new game for an existing instance.
    async fn resolve_install_plan(
        &self,
        params: ResolveInstallPlanParams,
    ) -> Result<InstallPlan, InstanceError> {
        let _ = params;
        Err(self.unsupported("resolve_install_plan"))
    }

    /// A pack's description without resolving its files: the preview before installing, and
    /// the check for updates (asked with [`PackSource::Content`], no version and the installed
    /// one, its [`PackMetadata::version_id`] compared with the installed one).
    ///
    /// A provider of single content has no packs to describe and keeps the default
    /// [`InstanceError::UnsupportedOperation`], which the UI reads as "no preview".
    async fn resolve_pack_metadata(
        &self,
        source: PackSource,
    ) -> Result<PackMetadata, InstanceError> {
        let _ = source;
        Err(self.unsupported("resolve_pack_metadata"))
    }

    /// What the provider says about each of `content_ids` — name, icon, author — to show
    /// installed content by, asked in one go for a whole instance.
    ///
    /// Each item is keyed by the provider's own id of the project, which may differ from the
    /// one asked by (a slug, for one). An id the provider does not know is left out, not an
    /// error. A provider that cannot describe its content keeps the default
    /// [`InstanceError::UnsupportedOperation`], and its content is shown by file alone.
    async fn get_contents_metadata(
        &self,
        content_ids: &[String],
    ) -> Result<Vec<ContentMetadata>, InstanceError> {
        let _ = content_ids;
        Err(self.unsupported("get_contents_metadata"))
    }

    async fn check_compatibility(
        &self,
        instance_ids: &[Instance],
        content_item: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError>;

    /// The error of a method this provider does not implement; `operation` is the method's name.
    fn unsupported(&self, operation: &'static str) -> InstanceError {
        InstanceError::UnsupportedOperation {
            provider_id: self.provider_id(),
            operation,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::features::instance::{CapabilityMetadata, ResolveTarget};

    use super::*;

    /// A provider that implements none of the resolve methods.
    struct BareProvider {
        metadata: ContentProviderCapabilityMetadata,
    }

    impl BareProvider {
        fn new() -> Self {
            Self {
                metadata: ContentProviderCapabilityMetadata {
                    base: CapabilityMetadata {
                        id: "bare".into(),
                        name: "Bare".into(),
                        description: None,
                        icon: None,
                    },
                    platform: None,
                },
            }
        }
    }

    #[async_trait]
    impl ContentProvider for BareProvider {
        fn metadata(&self) -> &ContentProviderCapabilityMetadata {
            &self.metadata
        }

        fn provider_id(&self) -> ProviderId {
            ProviderId {
                plugin_id: "plugin".into(),
                capability_id: self.metadata.base.id.clone(),
            }
        }

        async fn search(
            &self,
            _: ContentSearchParams,
        ) -> Result<ContentSearchResult, InstanceError> {
            unimplemented!()
        }

        async fn get_content(&self, _: String) -> Result<ContentItem, InstanceError> {
            unimplemented!()
        }

        async fn list_versions(&self, _: String) -> Result<Vec<ContentVersion>, InstanceError> {
            unimplemented!()
        }

        async fn check_compatibility(
            &self,
            _: &[Instance],
            _: &ContentCompatibilityCheckParams,
        ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
            unimplemented!()
        }
    }

    fn assert_unsupported(err: InstanceError, expected_operation: &str) {
        let InstanceError::UnsupportedOperation {
            provider_id,
            operation,
        } = err
        else {
            panic!("expected UnsupportedOperation, got {err:?}")
        };
        assert_eq!(provider_id.to_string(), "plugin#bare");
        assert_eq!(operation, expected_operation);
    }

    /// The default resolve methods answer with the provider's full id, and their futures can be
    /// spawned from behind `dyn ContentProvider` (they are `Send`).
    #[tokio::test]
    async fn should_report_resolve_methods_as_unsupported_when_the_provider_lacks_them() {
        let provider: Arc<dyn ContentProvider> = Arc::new(BareProvider::new());

        let file = tokio::spawn({
            let provider = provider.clone();
            async move {
                provider
                    .resolve_file("AANobbMI", None, "1.21.1", Some("fabric"))
                    .await
            }
        });
        let plan = tokio::spawn({
            let provider = provider.clone();
            async move {
                provider
                    .resolve_install_plan(ResolveInstallPlanParams {
                        content_id: "AANobbMI".into(),
                        version: None,
                        installed_version: None,
                        target: ResolveTarget::New,
                    })
                    .await
            }
        });
        let pack = tokio::spawn({
            let provider = provider.clone();
            async move {
                provider
                    .resolve_pack_metadata(PackSource::Url("https://example.com/pack.toml".into()))
                    .await
            }
        });

        assert_unsupported(file.await.unwrap().unwrap_err(), "resolve_file");
        assert_unsupported(plan.await.unwrap().unwrap_err(), "resolve_install_plan");
        assert_unsupported(pack.await.unwrap().unwrap_err(), "resolve_pack_metadata");
    }

    #[tokio::test]
    async fn should_report_contents_metadata_as_unsupported_when_the_provider_lacks_it() {
        let provider: Arc<dyn ContentProvider> = Arc::new(BareProvider::new());

        let metadata = tokio::spawn({
            let provider = provider.clone();
            async move {
                provider
                    .get_contents_metadata(&["AANobbMI".to_owned()])
                    .await
            }
        });

        assert_unsupported(
            metadata.await.unwrap().unwrap_err(),
            "get_contents_metadata",
        );
    }
}
