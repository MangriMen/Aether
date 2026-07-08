use crate::{
    core::app::AetherContainer,
    features::settings::{
        DefaultInstanceSettings, EditDefaultInstanceSettings, EditSettings, Settings,
        SettingsFeature,
    },
};

pub async fn get() -> crate::Result<Settings> {
    let container = AetherContainer::get();
    Ok(container.get_settings_use_case().execute().await?)
}

pub async fn edit(edit_settings: EditSettings) -> crate::Result<Settings> {
    let container = AetherContainer::get();
    Ok(container
        .edit_settings_use_case()
        .execute(edit_settings)
        .await?)
}

pub async fn get_default_instance_settings() -> crate::Result<DefaultInstanceSettings> {
    let container = AetherContainer::get();
    Ok(container
        .get_default_instance_settings_use_case()
        .execute()
        .await?)
}

pub async fn upsert_default_instance_settings(
    settings: EditDefaultInstanceSettings,
) -> crate::Result<DefaultInstanceSettings> {
    let container = AetherContainer::get();
    Ok(container
        .edit_default_instance_settings_use_case()
        .execute(settings)
        .await?)
}
