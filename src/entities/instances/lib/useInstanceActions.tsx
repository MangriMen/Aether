import { useContext } from 'solid-js';

import { isAetherLauncherError } from '@/shared/model';
import { Button, closeToast, showToast } from '@/shared/ui';

import type { Instance } from '@/entities/instances';
import {
  RunningInstancesContext,
  InstanceInstallStage,
  useLaunchInstance,
  useStopInstance,
  useRemoveInstance,
} from '@/entities/instances';

export const useInstanceActions = () => {
  const [context, { get: getRunningInstance, setIsLoading }] = useContext(
    RunningInstancesContext,
  );

  const { mutateAsync: launchInstance } = useLaunchInstance();
  const { mutateAsync: stopInstance } = useStopInstance();
  const { mutateAsync: removeInstance } = useRemoveInstance();

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

  const remove = async (instance: Instance, force: boolean = false) => {
    const runningInstance = getRunningInstance(context, instance.id);
    if (
      !force &&
      (instance.installStage !== InstanceInstallStage.Installed ||
        runningInstance?.isLoading ||
        runningInstance?.isRunning)
    ) {
      const handleForceRemove = async (e: MouseEvent) => {
        e.stopPropagation();
        await remove(instance, true);
        closeToast(id);
      };

      const id = showToast({
        title: `Failed to remove ${instance.name}`,
        description: (
          <div class='inline-flex w-full flex-col gap-2'>
            Instance still installing or running
            <Button
              class='w-full'
              variant='secondary'
              onClick={handleForceRemove}
              children='Remove force'
            />
          </div>
        ),
        variant: 'destructive',
      });
      return;
    }

    try {
      await removeInstance(instance.id);
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
  };
};
