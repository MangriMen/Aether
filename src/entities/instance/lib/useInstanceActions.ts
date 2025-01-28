import { useContext } from 'solid-js';

import { isAetherLauncherError, isDebug } from '@/shared/model';
import { showToast } from '@/shared/ui';

import type { Instance } from '@/entities/instance';
import {
  openInstanceFolder,
  refetchInstances,
  RunningInstancesContext,
  InstanceInstallStage,
  launchMinecraftInstance,
  removeMinecraftInstance,
  stopMinecraftInstance,
} from '@/entities/instance';

export const useInstanceActions = () => {
  const [context, { get: getRunningInstance, setIsLoading }] = useContext(
    RunningInstancesContext,
  );

  const launchInstance = async (instance: Instance) => {
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

  const stopInstance = async (instance: Instance) => {
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

  const openFolder = async (instance: Instance) => {
    try {
      await openInstanceFolder(instance);
    } catch (e) {
      if (isDebug()) {
        console.error(e);
      }

      showToast({
        title: 'Failed to open folder',
        description: `Instance path: ${instance.path}`,
        variant: 'destructive',
      });
    }
  };

  const removeInstance = async (instance: Instance) => {
    const runningInstance = getRunningInstance(context, instance.id);
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      runningInstance?.isLoading ||
      runningInstance?.isRunning
    ) {
      showToast({
        title: `Failed to remove ${instance.name}`,
        description: ' Still installing or running. Please stop instance first',
        variant: 'destructive',
      });
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
    launchInstance,
    stopInstance,
    removeInstance,
    openFolder,
  };
};
