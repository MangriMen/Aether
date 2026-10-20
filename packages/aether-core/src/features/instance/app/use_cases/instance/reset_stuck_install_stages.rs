use crate::features::instance::{InstanceError, InstanceInstallStage, InstanceStorage};

/// Moves instances that a crash or a kill left in a transient stage back to `NotInstalled`.
///
/// Must run before anything installs: a stage written by the previous process is stale by
/// definition, and `Installing` / `PackInstalling` would keep the instance unlaunchable for good.
/// Returns how many instances were reset.
pub async fn reset_stuck_install_stages(
    storage: &dyn InstanceStorage,
) -> Result<usize, InstanceError> {
    let mut reset = 0;
    for mut instance in storage.list().await? {
        if matches!(
            instance.install_stage,
            InstanceInstallStage::Installing | InstanceInstallStage::PackInstalling
        ) {
            log::warn!(
                "Instance \"{}\" was left in stage {:?}; resetting to NotInstalled",
                instance.name(),
                instance.install_stage
            );
            instance.install_stage = InstanceInstallStage::NotInstalled;
            storage.upsert(&instance).await?;
            reset += 1;
        }
    }
    Ok(reset)
}
