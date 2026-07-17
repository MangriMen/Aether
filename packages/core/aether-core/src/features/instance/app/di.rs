use std::sync::Arc;

use crate::features::instance::app::ports::{
    ChangeContentStateUseCasePort, CheckContentCompatibilityUseCasePort, ContentFileService,
    CreateInstanceUseCasePort, EditInstanceIconUseCasePort, EditInstanceUseCasePort,
    GetContentUseCasePort, GetInstanceUseCasePort, ImportContentUseCasePort,
    ImportInstanceUseCasePort, InstallContentUseCasePort, InstallContentV2UseCasePort,
    InstanceFileService,
    InstanceInstallService, InstanceLaunchService, InstanceWatcherService,
    LaunchInstanceWithActiveAccountUseCasePort, ListContentUseCasePort,
    ListContentVersionsUseCasePort, ListImportersUseCasePort, ListInstancesUseCasePort,
    ListProvidersUseCasePort, RemoveContentUseCasePort, RemoveInstanceUseCasePort,
    SearchContentUseCasePort, UpdateInstanceUseCasePort,
};

pub trait InstanceCrudPort {
    fn create_instance_use_case(&self) -> Arc<dyn CreateInstanceUseCasePort>;
    fn get_instance_use_case(&self) -> Arc<dyn GetInstanceUseCasePort>;
    fn list_instances_use_case(&self) -> Arc<dyn ListInstancesUseCasePort>;
    fn edit_instance_use_case(&self) -> Arc<dyn EditInstanceUseCasePort>;
    fn edit_instance_icon_use_case(&self) -> Arc<dyn EditInstanceIconUseCasePort>;
    fn remove_instance_use_case(&self) -> Arc<dyn RemoveInstanceUseCasePort>;
    fn update_instance_use_case(&self) -> Arc<dyn UpdateInstanceUseCasePort>;
}

pub trait InstanceLifecyclePort {
    fn import_instance_use_case(&self) -> Arc<dyn ImportInstanceUseCasePort>;
    fn list_importers_use_case(&self) -> Arc<dyn ListImportersUseCasePort>;
    fn launch_instance_with_active_account_use_case(
        &self,
    ) -> Arc<dyn LaunchInstanceWithActiveAccountUseCasePort>;
}

pub trait ContentManagementPort {
    fn change_content_state_use_case(&self) -> Arc<dyn ChangeContentStateUseCasePort>;
    fn import_content_use_case(&self) -> Arc<dyn ImportContentUseCasePort>;
    fn list_content_use_case(&self) -> Arc<dyn ListContentUseCasePort>;
    fn remove_content_use_case(&self) -> Arc<dyn RemoveContentUseCasePort>;
}

pub trait ContentProviderPort {
    fn search_content_use_case(&self) -> Arc<dyn SearchContentUseCasePort>;
    fn get_content_use_case(&self) -> Arc<dyn GetContentUseCasePort>;
    fn install_content_use_case(&self) -> Arc<dyn InstallContentUseCasePort>;
    fn install_content_v2_use_case(&self) -> Arc<dyn InstallContentV2UseCasePort>;
    fn check_content_compatibility_use_case(&self)
    -> Arc<dyn CheckContentCompatibilityUseCasePort>;
    fn list_content_versions_use_case(&self) -> Arc<dyn ListContentVersionsUseCasePort>;
    fn list_providers_use_case(&self) -> Arc<dyn ListProvidersUseCasePort>;
}

pub trait InstanceServicesPort {
    fn instance_install_service(&self) -> Arc<dyn InstanceInstallService>;
    fn instance_launch_service(&self) -> Arc<dyn InstanceLaunchService>;
    fn instance_watcher_service(&self) -> Arc<dyn InstanceWatcherService>;
    fn instance_file_service(&self) -> Arc<dyn InstanceFileService>;
    fn content_file_service(&self) -> Arc<dyn ContentFileService>;
}

/// Super-trait combining all instance sub-traits for convenience.
///
/// Consumers that need only a subset should depend on the
/// specific sub-trait instead.
pub trait InstanceFeature:
    InstanceCrudPort
    + InstanceLifecyclePort
    + ContentManagementPort
    + ContentProviderPort
    + InstanceServicesPort
{
}

impl<T> InstanceFeature for T where
    T: InstanceCrudPort
        + InstanceLifecyclePort
        + ContentManagementPort
        + ContentProviderPort
        + InstanceServicesPort
{
}
