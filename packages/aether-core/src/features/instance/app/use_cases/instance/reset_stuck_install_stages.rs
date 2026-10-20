use crate::features::instance::{InstanceError, InstanceInstallStage, InstanceStorage};

/// Moves instances that a crash or a kill left in a transient stage back to `NotInstalled`,
/// where the user can repair them.
///
/// Must run before anything installs: a stage written by the previous process is stale by
/// definition, and `Installing` / `PackInstalling` would keep the instance unlaunchable for good.
/// That includes an installed instance whose reinstall was cut short: the stage it had before
/// is not kept, so it comes back `NotInstalled` and is repaired from there. A pack's instance
/// keeps its game but may lack content: updating the pack deploys it again.
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
            tracing::warn!(
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
