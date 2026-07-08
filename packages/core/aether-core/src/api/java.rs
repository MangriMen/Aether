use crate::{
    core::app::AetherContainer,
    features::java::{InstallJava, Java, JavaFeature},
};

#[tracing::instrument]
pub async fn install(version: u32) -> crate::Result<Java> {
    let container = AetherContainer::get();
    Ok(container
        .install_java_use_case()
        .execute(InstallJava::new(version))
        .await?)
}

pub async fn get(version: u32) -> crate::Result<Java> {
    let container = AetherContainer::get();
    Ok(container.get_java_use_case().execute(version).await?)
}
