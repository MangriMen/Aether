import { createMemo } from 'solid-js';

import { useInstances, useRunningInstancesContext } from '@/features/instance';

import { getRunningGameNames } from '../model/runningGames';

/**
 * The games that run. The launcher restarts after an update, so while there are any the update
 * is not installed: the games would be left without it.
 */
export const useRunningGames = () => {
  const [running] = useRunningInstancesContext();
  const instances = useInstances();

  const names = createMemo(() =>
    getRunningGameNames(running.instances, instances.data),
  );

  return { names, isRunning: () => names().length > 0 };
};
