import { useContext } from 'solid-js';

import { isAetherLauncherError, isDebug } from '@/shared/model';
import { showToast } from '@/shared/ui';

import type { Instance } from '@/entities/instances';
import {
  revealInExplorer,
  refetchInstances,
  RunningInstancesContext,
  InstanceInstallStage,
  launchInstance,
  removeInstance,
  stopInstance,
} from '@/entities/instances';

export const useInstanceActions = () => {
  const [context, { get: getRunningInstance, setIsLoading }] = useContext(
    RunningInstancesContext,
  );

  const launch = async (instance: Instance) => {
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
      await launchInstance(instance.id);
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

  const stop = async (instance: Instance) => {
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
      await stopInstance(uuid);
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
      await revealInExplorer(instance.path);
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

  const remove = async (instance: Instance) => {
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
      await removeInstance(instance.id);
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
    launch,
    stop,
    remove,
    openFolder,
  };
};
