import { useContext } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Button, closeToast, showToast } from '@/shared/ui-kit';

import type { Instance } from '../model/instance';

import { useLaunchInstance, useStopInstance } from '../api/instanceLaunch';
import { useRemoveInstance } from '../api/instanceQueries';
import {
  getRemovalBlock,
  isStoppable,
  type RemovalBlock,
} from '../model/runningInstances';
import { RunningInstancesContext } from '../model/runningInstancesContext';

export const useInstanceActions = () => {
  const [context, { get: getRunningInstance, setIsLoading, refresh }] =
    useContext(RunningInstancesContext);

  const launchMutation = useLaunchInstance();
  const { mutateAsync: stopInstance } = useStopInstance();
  const { mutateAsync: removeInstance } = useRemoveInstance();

  const [{ t }] = useTranslation();

  const launch = async (instance: Instance) => {
    const runningInstance = getRunningInstance(context, instance.id);

    if (
      instance.installStage !== 'installed' ||
      runningInstance?.isLoading ||
      runningInstance?.isRunning ||
      // The backend has not told yet that this launch began: a second click must not start one more.
      launchMutation.isPending
    ) {
      return;
    }

    // The backend tells that the launch has begun and that it is over.
    try {
      await launchMutation.mutateAsync(instance.id);
    } catch {
      /* empty */
    }
  };

  /** Stops the game of the instance, or calls its launch off while it has no game yet. */
  const stop = async (instance: Instance) => {
    if (!isStoppable(getRunningInstance(context, instance.id))) {
      return;
    }

    try {
      await stopInstance(instance.id);
    } catch {
      /* empty */
    }
  };

  const showRemovalBlocked = (instance: Instance, block: RemovalBlock) => {
    const title = (
      <span class='line-clamp-2'>
        {t('instance.failedToRemove', { name: instance.name })}
      </span>
    );

    if (block === 'running' || block === 'launching') {
      // Stops the game, or calls the launch off while it has no game yet.
      const handleStop = async (e: MouseEvent) => {
        e.stopPropagation();
        await stop(instance);

        if (block === 'running') {
          await refresh();
        }

        closeToast(id);
      };

      const id = showToast({
        title,
        description: (
          <div class='gap-2 inline-flex w-full flex-col'>
            {block === 'running'
              ? t('instance.stopToDelete')
              : t('instance.removeWhileLaunching')}
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
