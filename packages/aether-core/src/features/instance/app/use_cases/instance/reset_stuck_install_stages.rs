use crate::features::instance::{InstanceError, InstanceInstallStage, InstanceStorage};

/// Moves instances that a crash or a kill left in `Installing` back to `NotInstalled`.
///
/// Must run before anything installs: a stage written by the previous process is stale by
/// definition, and `Installing` would keep the instance unlaunchable for good. `PackInstalling`
/// is left as it is: it is unlaunchable on purpose until its pack is deployed, and resetting it
/// would make a half-deployed pack launchable. Returns how many instances were reset.
pub async fn reset_stuck_install_stages(
    storage: &dyn InstanceStorage,
) -> Result<usize, InstanceError> {
    let mut reset = 0;
    for mut instance in storage.list().await? {
        if instance.install_stage == InstanceInstallStage::Installing {
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
