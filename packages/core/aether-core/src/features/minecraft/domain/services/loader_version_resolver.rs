use std::sync::Arc;

use crate::features::minecraft::{
    LoaderVersionPreference, MetadataStorage, MinecraftDomainError, ModLoader,
};

pub struct LoaderVersionResolver<MS> {
    metadata_storage: Arc<MS>,
}

impl<MS: MetadataStorage> LoaderVersionResolver<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }

    pub async fn resolve(
        &self,
        game_version: &str,
        loader: &ModLoader,
        loader_version: Option<&LoaderVersionPreference>,
    ) -> Result<Option<daedalus::modded::LoaderVersion>, MinecraftDomainError> {
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

    pub async fn try_get_default(
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

pub async fn resolve_loader_version(
    game_version: &str,
    loader: &ModLoader,
    loader_version_preference: &LoaderVersionPreference,
    loader_version_manifest: &daedalus::modded::Manifest,
) -> Result<Option<daedalus::modded::LoaderVersion>, MinecraftDomainError> {
    if matches!(loader, ModLoader::Vanilla) {
        return Ok(None);
    }

    let found_game_version = loader_version_manifest
        .game_versions
        .iter()
        .find(|x| {
            x.id.replace(daedalus::modded::DUMMY_REPLACE_STRING, game_version) == game_version
        })
        .ok_or(MinecraftDomainError::VersionForLoaderNotFound {
            loader_version_preference: loader_version_preference.clone(),
        })?;

    Ok(Some(
        find_loader_version(&found_game_version.loaders, loader_version_preference).cloned()?,
    ))
}

fn find_loader_version<'a>(
    loaders: &'a [daedalus::modded::LoaderVersion],
    preference: &LoaderVersionPreference,
) -> Result<&'a daedalus::modded::LoaderVersion, MinecraftDomainError> {
    match preference {
        LoaderVersionPreference::Latest => loaders.first(),
        LoaderVersionPreference::Stable => loaders.iter().find(|x| x.stable),
        LoaderVersionPreference::Exact(id) => loaders.iter().find(|x| x.id == *id),
    }
    .ok_or(MinecraftDomainError::LoaderNotFound {
        loader_version_preference: preference.clone(),
    })
}
