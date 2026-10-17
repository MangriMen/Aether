use crate::features::minecraft::{
    LoaderVersionPreference, LoaderVersionService, MinecraftDomainError, ModLoader, modded,
};

use super::super::ports::MetadataStorage;

use std::sync::Arc;

pub struct LoaderVersionResolver {
    metadata_storage: Arc<dyn MetadataStorage>,
}

#[async_trait::async_trait]
impl LoaderVersionService for LoaderVersionResolver {
    async fn resolve(
        &self,
        game_version: &str,
        loader: &ModLoader,
        loader_version: Option<&LoaderVersionPreference>,
    ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
        if matches!(loader, ModLoader::Vanilla) {
            return Ok(None);
        }

        let Some(loader_version) = loader_version else {
            return Err(MinecraftDomainError::LoaderVersionRequired);
        };

        let loader_version_manifest = self
            .metadata_storage
            .get_loader_version_manifest(*loader)
            .await?;

        resolve_loader_version(
            game_version,
            loader,
            loader_version,
            &loader_version_manifest,
        )
        .await
    }

    async fn try_get_default(
        &self,
        game_version: &str,
        loader: &ModLoader,
    ) -> Result<Option<LoaderVersionPreference>, MinecraftDomainError> {
        if matches!(loader, ModLoader::Vanilla) {
            return Ok(None);
        }

        let loader_version_manifest = self
            .metadata_storage
            .get_loader_version_manifest(*loader)
            .await?;

        let default_preferences = [
            LoaderVersionPreference::Stable,
            LoaderVersionPreference::Latest,
        ];

        for preference in default_preferences {
            if resolve_loader_version(game_version, loader, &preference, &loader_version_manifest)
                .await
                .is_ok()
            {
                return Ok(Some(preference));
            }
        }

        Err(MinecraftDomainError::DefaultLoaderNotFound)
    }
}

impl LoaderVersionResolver {
    pub fn new(metadata_storage: Arc<dyn MetadataStorage>) -> Self {
        Self { metadata_storage }
    }

    pub async fn resolve(
        &self,
        game_version: &str,
        loader: &ModLoader,
        loader_version: Option<&LoaderVersionPreference>,
    ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
        LoaderVersionService::resolve(self, game_version, loader, loader_version).await
    }

    pub async fn try_get_default(
        &self,
        game_version: &str,
        loader: &ModLoader,
    ) -> Result<Option<LoaderVersionPreference>, MinecraftDomainError> {
        LoaderVersionService::try_get_default(self, game_version, loader).await
    }
}

pub(super) async fn resolve_loader_version(
    game_version: &str,
    loader: &ModLoader,
    loader_version_preference: &LoaderVersionPreference,
    loader_version_manifest: &modded::Manifest,
) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
    if matches!(loader, ModLoader::Vanilla) {
        return Ok(None);
    }

    let found_game_version = loader_version_manifest
        .game_versions
        .iter()
        .find(|x| x.id.replace(modded::DUMMY_REPLACE_STRING, game_version) == game_version)
        .ok_or(MinecraftDomainError::VersionForLoaderNotFound {
            loader_version_preference: loader_version_preference.clone(),
        })?;

    Ok(Some(
        find_loader_version(
            loader_version_manifest
                .loaders_of(found_game_version)
                .ok_or(MinecraftDomainError::VersionForLoaderNotFound {
                    loader_version_preference: loader_version_preference.clone(),
                })?,
            loader_version_preference,
        )
        .cloned()?,
    ))
}

fn find_loader_version<'a>(
    loaders: &'a [modded::LoaderVersion],
    preference: &LoaderVersionPreference,
) -> Result<&'a modded::LoaderVersion, MinecraftDomainError> {
    match preference {
        LoaderVersionPreference::Latest => loaders.first(),
        LoaderVersionPreference::Stable => loaders.iter().find(|x| x.stable),
        LoaderVersionPreference::Exact(id) => loaders.iter().find(|x| x.id == *id),
    }
    .ok_or(MinecraftDomainError::LoaderNotFound {
        loader_version_preference: preference.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loader(id: &str, stable: bool) -> modded::LoaderVersion {
        modded::LoaderVersion {
            id: id.to_string(),
            url: format!("https://meta/{id}"),
            stable,
        }
    }

    fn grouped_manifest() -> modded::Manifest {
        modded::Manifest {
            game_versions: vec![
                modded::Version {
                    id: "1.21".to_string(),
                    stable: true,
                    loaders: vec![],
                    version_group: Some("v1".to_string()),
                },
                modded::Version {
                    id: "26.3".to_string(),
                    stable: true,
                    loaders: vec![],
                    version_group: Some("v2".to_string()),
                },
                modded::Version {
                    id: "1.8".to_string(),
                    stable: true,
                    loaders: vec![loader("own", true)],
                    version_group: None,
                },
                modded::Version {
                    id: "1.7".to_string(),
                    stable: true,
                    loaders: vec![],
                    version_group: Some("missing".to_string()),
                },
            ],
            version_groups: vec![
                modded::VersionGroup {
                    id: "v1".to_string(),
                    loaders: vec![loader("0.30.0-beta", false), loader("0.29.0", true)],
                },
                modded::VersionGroup {
                    id: "v2".to_string(),
                    loaders: vec![loader("0.31.0", true)],
                },
            ],
        }
    }

    async fn resolve(
        game_version: &str,
        preference: LoaderVersionPreference,
    ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
        resolve_loader_version(
            game_version,
            &ModLoader::Quilt,
            &preference,
            &grouped_manifest(),
        )
        .await
    }

    #[tokio::test]
    async fn should_take_loaders_from_the_group_when_a_game_version_has_one() {
        let latest = resolve("1.21", LoaderVersionPreference::Latest).await;
        let stable = resolve("1.21", LoaderVersionPreference::Stable).await;
        let other_group = resolve("26.3", LoaderVersionPreference::Latest).await;

        assert_eq!(latest.unwrap().unwrap().id, "0.30.0-beta");
        assert_eq!(stable.unwrap().unwrap().id, "0.29.0");
        assert_eq!(other_group.unwrap().unwrap().id, "0.31.0");
    }

    #[tokio::test]
    async fn should_take_own_loaders_when_a_game_version_has_no_group() {
        let resolved = resolve("1.8", LoaderVersionPreference::Latest).await;

        assert_eq!(resolved.unwrap().unwrap().id, "own");
    }

    #[tokio::test]
    async fn should_fail_when_the_group_of_a_game_version_does_not_exist() {
        let resolved = resolve("1.7", LoaderVersionPreference::Latest).await;

        assert!(matches!(
            resolved,
            Err(MinecraftDomainError::VersionForLoaderNotFound { .. })
        ));
    }
}
