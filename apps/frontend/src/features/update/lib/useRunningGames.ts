import { createMemo } from 'solid-js';

import { useInstances, useRunningInstancesContext } from '@/features/instance';

import { useGamesCarryOnUpdate } from '../api/updateQueries';
import { getRunningGameNames } from '../model/runningGames';

/**
 * The games that run. The launcher restarts after an update, so while there are any the update
 * is not installed, unless they all carry on across the restart (`keepsGames`): the next launcher
 * follows them.
 */
export const useRunningGames = () => {
  const [running] = useRunningInstancesContext();
  const instances = useInstances();

  const names = createMemo(() =>
    getRunningGameNames(running.instances, instances.data),
  );

  const carryOn = useGamesCarryOnUpdate(names);

  // A launch is in the gate: the install is refused until it is over.
  const isLaunching = () =>
    Object.values(running.instances).some(
      (state) => state?.isLoading && !state.isRunning,
    );

  const isRunning = () => names().length > 0;

  const keepsGames = () =>
    isRunning() && !isLaunching() && carryOn.data === true;

  return {
    names,
    isRunning,
    keepsGames,
    /** A game would be lost by the restart. */
    isBlocked: () => isRunning() && !keepsGames(),
  };
};
