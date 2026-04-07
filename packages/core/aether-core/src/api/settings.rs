use crate::{
    core::domain::LazyLocator,
    features::settings::{
        app::{
            EditDefaultInstanceSettings, EditDefaultInstanceSettingsUseCase, EditSettings,
            EditSettingsUseCase, GetDefaultInstanceSettingsUseCase, GetSettingsUseCase,
        },
        DefaultInstanceSettings, Settings,
    },
};

pub async fn get() -> crate::Result<Settings> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetSettingsUseCase::new(lazy_locator.get_settings_storage().await)
            .execute()
            .await?,
    )
}

pub async fn edit(edit_settings: EditSettings) -> crate::Result<Settings> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        EditSettingsUseCase::new(lazy_locator.get_settings_storage().await)
            .execute(edit_settings)
            .await?,
    )
}

pub async fn get_default_instance_settings() -> crate::Result<DefaultInstanceSettings> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(GetDefaultInstanceSettingsUseCase::new(
        lazy_locator.get_default_instance_settings_storage().await,
    )
    .execute()
    .await?)
}

pub async fn upsert_default_instance_settings(
    settings: EditDefaultInstanceSettings,
) -> crate::Result<DefaultInstanceSettings> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(EditDefaultInstanceSettingsUseCase::new(
        lazy_locator.get_default_instance_settings_storage().await,
    )
    .execute(settings)
    .await?)
}
