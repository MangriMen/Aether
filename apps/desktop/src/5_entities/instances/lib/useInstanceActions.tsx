import { useContext } from 'solid-js';

import type { Instance } from '@/entities/instances';

import {
  RunningInstancesContext,
  InstanceInstallStage,
  useLaunchInstance,
  useStopInstance,
  useRemoveInstance,
} from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { Button, closeToast, showToast } from '@/shared/ui';

export const useInstanceActions = () => {
  const [context, { get: getRunningInstance, setIsLoading }] = useContext(
    RunningInstancesContext,
  );

  const { mutateAsync: launchInstance } = useLaunchInstance();
  const { mutateAsync: stopInstance } = useStopInstance();
  const { mutateAsync: removeInstance } = useRemoveInstance();

  const [{ t }] = useTranslation();

  const launch = async (instance: Instance) => {
    const runningInstance = getRunningInstance(context, instance.id);
    if (
      instance.installStage !== InstanceInstallStage.Installed ||
      runningInstance?.isLoading ||
      runningInstance?.isRunning
    ) {
      return;
    }

    setIsLoading(instance.id, true);
    try {
      await launchInstance(instance.id);
    } catch {
      /* empty */
    }
    setIsLoading(instance.id, false);
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

    const uuid = runningInstance.payload?.processId;
    if (uuid === undefined) {
      return;
    }

    setIsLoading(instance.id, true);
    try {
      await stopInstance(uuid);
    } catch {
      /* empty */
    }
    setIsLoading(instance.id, false);
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
        title: (
          <span class='line-clamp-2'>
            {t('instance.failedToRemove', { name: instance.name })}
          </span>
        ),
        description: (
          <div class='inline-flex w-full flex-col gap-2'>
            {t('instance.failedToRemoveDescription')}
            <Button
              class='w-full'
              variant='secondary'
              onClick={handleForceRemove}
              children={t('instance.removeForce')}
            />
          </div>
        ),
        variant: 'destructive',
      });
      return;
    }

    setIsLoading(instance.id, true);
    try {
      await removeInstance(instance.id);
    } catch {
      /* empty */
    }
    setIsLoading(instance.id, false);
  };

  return {
    launch,
    stop,
    remove,
  };
};
