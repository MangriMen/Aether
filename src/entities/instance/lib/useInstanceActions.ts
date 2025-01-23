import { useContext } from 'solid-js';

import { isAetherLauncherError } from '@/shared/model';
import { showToast } from '@/shared/ui';

import {
  openInstanceFolder,
  refetchInstances,
  RunningInstancesContext,
  Instance,
  InstanceInstallStage,
  launchMinecraftInstance,
  removeMinecraftInstance,
  stopMinecraftInstance,
} from '@/entities/instance';

export const useInstanceActions = () => {
  const [context, { get: getRunningInstance, setIsLoading }] = useContext(
    RunningInstancesContext,
  );

  const handleInstanceLaunch = async (instance: Instance) => {
    const runningInstance = getRunningInstance(context, instance.id);
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      runningInstance?.isLoading ||
      runningInstance?.isRunning
    ) {
      return;
    }

    try {
      setIsLoading(instance.id, true);
      await launchMinecraftInstance(instance.id);
    } catch (e) {
      setIsLoading(instance.id, false);
      if (isAetherLauncherError(e)) {
        showToast({
          title: `Failed to launch ${instance.name}`,
          description: e.message,
          variant: 'destructive',
        });
      }
    }
  };

  const handleInstanceStop = async (instance: Instance) => {
    const runningInstance = getRunningInstance(context, instance.id);
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      !runningInstance ||
      runningInstance.isLoading
    ) {
      return;
    }

    const uuid = runningInstance.payload?.uuid;
    if (uuid === undefined) {
      return;
    }

    try {
      setIsLoading(instance.id, true);
      await stopMinecraftInstance(uuid);
    } catch (e) {
      setIsLoading(instance.id, false);
      if (isAetherLauncherError(e)) {
        showToast({
          title: `Failed to stop ${instance.name}`,
          description: e.message,
          variant: 'destructive',
        });
      }
    }
  };

  const handleOpenFolder = async (instance: Instance) => {
    try {
      await openInstanceFolder(instance);
    } catch (e) {
      showToast({
        title: 'Failed to open folder',
        description: `Instance path: ${instance.path}`,
        variant: 'destructive',
      });
    }
  };

  const handleInstanceRemove = async (instance: Instance) => {
    const runningInstance = getRunningInstance(context, instance.id);
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      !runningInstance ||
      runningInstance.isLoading ||
      runningInstance.isRunning
    ) {
      return;
    }

    try {
      await removeMinecraftInstance(instance.id);
      refetchInstances();
    } catch (e) {
      if (isAetherLauncherError(e)) {
        showToast({
          title: `Failed to remove ${instance.name}`,
          description: e.message,
          variant: 'destructive',
        });
      }
    }
  };

  return {
    handleInstanceLaunch,
    handleInstanceStop,
    handleInstanceRemove,
    handleOpenFolder,
  };
};
