use crate::{
    core::app::AetherContainer,
    features::instance::{EditInstance, EditInstanceIcon, Instance, InstanceFeature, NewInstance},
};

#[tracing::instrument]
pub async fn create(new_instance: NewInstance) -> crate::Result<String> {
    let container = AetherContainer::get();
    Ok(container
        .create_instance_use_case()
        .execute(new_instance)
        .await?)
}

#[tracing::instrument]
pub async fn install(instance_id: String, force: bool) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .instance_install_service()
        .execute(instance_id, force)
        .await?)
}

#[tracing::instrument]
pub async fn update(instance_id: String) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .update_instance_use_case()
        .execute(instance_id)
        .await?)
}

pub async fn list() -> crate::Result<Vec<Instance>> {
    let container = AetherContainer::get();
    Ok(container.list_instances_use_case().execute().await?)
}

pub async fn get(instance_id: String) -> crate::Result<Instance> {
    let container = AetherContainer::get();
    Ok(container
        .get_instance_use_case()
        .execute(instance_id)
        .await?)
}

#[tracing::instrument]
pub async fn edit(instance_id: String, edit_instance: EditInstance) -> crate::Result<Instance> {
    let container = AetherContainer::get();
    Ok(container
        .edit_instance_use_case()
        .execute(instance_id, edit_instance)
        .await?)
}

#[tracing::instrument]
pub async fn edit_icon(
    instance_id: String,
    icon_path: Option<Option<String>>,
) -> crate::Result<Instance> {
    let container = AetherContainer::get();
    Ok(container
        .edit_instance_icon_use_case()
        .execute(EditInstanceIcon {
            instance_id,
            icon_path,
        })
        .await?)
}

#[tracing::instrument]
pub async fn remove(instance_id: String) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .remove_instance_use_case()
        .execute(instance_id)
        .await?)
}
