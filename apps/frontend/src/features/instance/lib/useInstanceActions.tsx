import { useContext } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Button, closeToast, showToast } from '@/shared/ui-kit';

import type { Instance } from '../model/instance';

import { useLaunchInstance, useStopInstance } from '../api/instanceLaunch';
import { useRemoveInstance } from '../api/instanceQueries';
import {
  getRemovalBlock,
  getStoppableProcessId,
  type RemovalBlock,
} from '../model/runningInstances';
import { RunningInstancesContext } from '../model/runningInstancesContext';

export const useInstanceActions = () => {
  const [context, { get: getRunningInstance, setIsLoading, refresh }] =
    useContext(RunningInstancesContext);

  const { mutateAsync: launchInstance } = useLaunchInstance();
  const { mutateAsync: stopInstance } = useStopInstance();
  const { mutateAsync: removeInstance } = useRemoveInstance();

  const [{ t }] = useTranslation();

  const launch = async (instance: Instance) => {
    const runningInstance = getRunningInstance(context, instance.id);

    if (
      instance.installStage !== 'installed' ||
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
    const uuid = getStoppableProcessId(
      getRunningInstance(context, instance.id),
    );

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

  const showRemovalBlocked = (instance: Instance, block: RemovalBlock) => {
    const title = (
      <span class='line-clamp-2'>
        {t('instance.failedToRemove', { name: instance.name })}
      </span>
    );

    if (block === 'running') {
      const handleStop = async (e: MouseEvent) => {
        e.stopPropagation();
        await stop(instance);
        await refresh();
        closeToast(id);
      };

      const id = showToast({
        title,
        description: (
          <div class='gap-2 inline-flex w-full flex-col'>
            {t('instance.stopToDelete')}
            <Button
              class='w-full'
              variant='secondary'
              onClick={handleStop}
              children={t('instance.stop')}
            />
          </div>
        ),
        variant: 'destructive',
      });
      return;
    }

    if (block === 'launching') {
      showToast({
        title,
        description: t('instance.removeWhileLaunching'),
        variant: 'destructive',
      });
      return;
    }

    const handleForceRemove = async (e: MouseEvent) => {
      e.stopPropagation();
      await remove(instance, true);
      closeToast(id);
    };

    const id = showToast({
      title,
      description: (
        <div class='gap-2 inline-flex w-full flex-col'>
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
  };

  const currentRemovalBlock = (instance: Instance) =>
    getRemovalBlock(
      instance.installStage,
      getRunningInstance(context, instance.id),
    );

  /** Tells why a removal that cannot go on is refused; false when it may be asked for. */
  const notifyRemovalBlocked = (instance: Instance): boolean => {
    const block = currentRemovalBlock(instance);

    // A not fully installed instance is offered the force removal once the removal is confirmed.
    if (block === undefined || block === 'notInstalled') {
      return false;
    }

    showRemovalBlocked(instance, block);
    return true;
  };

  const remove = async (instance: Instance, force: boolean = false) => {
    const block = currentRemovalBlock(instance);

    // Force only overrides an incomplete install, never a running or launching game.
    if (block !== undefined && (block !== 'notInstalled' || !force)) {
      showRemovalBlocked(instance, block);
      return;
    }

    setIsLoading(instance.id, true);

    try {
      await removeInstance(instance.id);
    } catch {
      // The game may have started behind the UI's back; the mutation shows the error.
      await refresh();
    }

    setIsLoading(instance.id, false);
  };

  return {
    launch,
    stop,
    remove,
    notifyRemovalBlocked,
  };
};
