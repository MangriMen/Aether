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
    for listed in storage.list().await? {
        let mut was_stuck = None;
        storage
            .update(
                listed.id(),
                Box::new(|instance| {
                    if !matches!(
                        instance.install_stage,
                        InstanceInstallStage::Installing | InstanceInstallStage::PackInstalling
                    ) {
                        return Ok(false);
                    }
                    was_stuck = Some(instance.install_stage);
                    instance.install_stage = InstanceInstallStage::NotInstalled;
                    Ok(true)
                }),
            )
            .await?;
        if let Some(stage) = was_stuck {
            tracing::warn!(
                "Instance \"{}\" was left in stage {stage:?}; reset to NotInstalled",
                listed.name()
            );
            reset += 1;
        }
    }
    Ok(reset)
}
