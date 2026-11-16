import { useContext } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Button, closeToast, showToast } from '@/shared/ui-kit';

import type { Instance } from '../model/instance';

import {
  useCancelLaunch,
  useLaunchInstance,
  useStopInstance,
} from '../api/instanceLaunch';
import { useRemoveInstance } from '../api/instanceQueries';
import {
  getRemovalBlock,
  getStopAction,
  type RemovalBlock,
} from '../model/runningInstances';
import { RunningInstancesContext } from '../model/runningInstancesContext';

/**
 * The instances whose launch was started by this page, so that a `launch` is waiting to see it
 * end. A launch that is not here was recovered after a reload: nothing waits for it.
 */
const ownLaunches = new Set<string>();

export const useInstanceActions = () => {
  const [
    context,
    { get: getRunningInstance, setIsLoading, setLaunch, refresh },
  ] = useContext(RunningInstancesContext);

  const { mutateAsync: launchInstance } = useLaunchInstance();
  const { mutateAsync: stopInstance } = useStopInstance();
  const { mutateAsync: cancelLaunch } = useCancelLaunch();
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

    setLaunch(instance.id, 'launching');
    ownLaunches.add(instance.id);

    try {
      const process = await launchInstance(instance.id);

      // The call off came too late for the launch, and the game started all the same.
      if (getRunningInstance(context, instance.id)?.launch === 'cancelling') {
        await stopInstance(process.uuid);
      }
    } catch {
      /* empty */
    }

    ownLaunches.delete(instance.id);
    setLaunch(instance.id, undefined);
  };

  /** Stops the game of the instance, or calls its launch off while it has no game yet. */
  const stop = async (instance: Instance) => {
    const action = getStopAction(getRunningInstance(context, instance.id));

    if (action === undefined) {
      return;
    }

    if (action.kind === 'cancelLaunch') {
      // The launch itself ends it: with `Cancelled` when it was called off in time, or with the
      // game, which `launch` then stops. The state stays until then, so that Play is not offered
      // while the launch still holds the instance.
      setLaunch(instance.id, 'cancelling');

      try {
        const found = await cancelLaunch(instance.id);

        // No launch to call off: the game is starting. A launch of this page stops it when it
        // returns; a recovered one has nobody to, and is left to the user to stop.
        if (!found && !ownLaunches.has(instance.id)) {
          setLaunch(instance.id, undefined);
        }
      } catch {
        setLaunch(instance.id, 'launching');
      }

      return;
    }

    setIsLoading(instance.id, true);

    try {
      await stopInstance(action.processId);
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
